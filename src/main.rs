pub mod bin_modules;
use bin_modules::cli::Command;
use bin_modules::{Cache, DestinyFetchError, Result, cli, database, download, search, sync};
use chrono::{DateTime, TimeDelta, Utc};
use clap::{CommandFactory, Parser};
use dirs;
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::{Sqlite, SqlitePool};
use std::path::Path;
use std::{fs, path::PathBuf};

pub static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!();

#[tokio::main]
async fn main() {
    let _cli: cli::CLI = cli::CLI::parse();
    let mut cache: Cache = match Cache::open_default(true) {
        Ok(cache) => cache,
        Err(_) => {
            println!("Unable to access cache directory");
            return;
        }
    };

    let db_path: PathBuf = match database::get_db_path() {
        Ok(path) => path,
        Err(_) => {
            println!("Unable to access database directory");
            return;
        }
    };

    let pool = SqlitePool::connect_lazy_with(
        SqliteConnectOptions::default()
            .foreign_keys(false)
            .filename(&db_path),
    );

    /* Conditions that will automatically trigger a database sync
        1. cache file does not exist
        2. database file does not exist
        3. either cache value for last_sync_at and/or last_sync_rows_written is null
        4. cache timestamp for last_sync_at is two or more weeks old
        5. cache value for last_sync_rows_written does not match the total number of rows in the current database
    */

    let needs_sync = sync::database_needs_synced(pool.clone(), &db_path, &cache).await;

    if needs_sync | matches!(&_cli.cmd, Command::Sync) {
        match sync_destinypedia(&mut cache, &db_path).await {
            Ok(()) => {
                println!("Database sync successful!")
            }
            Err(e) => {
                println!(
                    "ERROR: Failed to sync database with error: {}",
                    e.to_string()
                );
                return;
            }
        }
    }

    match _cli.cmd {
        Command::Search(args) => match search::search(&args, pool).await {
            Ok(_) => {
                println!("Search successful!")
            }
            Err(DestinyFetchError::Quit) => {
                println!("Unable to complete search due to early shutdown")
            }
            Err(e) => {
                println!(
                    "ERROR: Failed to complete search with error: {}",
                    e.to_string()
                );
            }
        },
        Command::Download(args) => match download::download(args, pool).await {
            Ok(()) => {
                println!("Download successful!")
            }
            Err(DestinyFetchError::Quit) => {
                println!("Unable to complete download due to early shutdown")
            }
            Err(e) => {
                println!(
                    "ERROR: Failed to complete download with error: {}",
                    e.to_string()
                );
            }
        },
        _ => (),
    }
}

async fn sync_destinypedia(cache: &mut Cache, db: impl AsRef<Path>) -> Result<()> {
    // create temporary database as the write target for sync
    let tmp = db.as_ref().to_path_buf().with_added_extension("tmp");

    let backup: Option<PathBuf> = {
        if fs::exists(&db)? {
            // if a database already exists rename as backup
            sync::create_backup(&db).ok()
        } else {
            // otherwise do nothing
            None
        }
    };

    let pool = SqlitePool::connect_lazy_with(
        SqliteConnectOptions::default()
            .foreign_keys(false)
            .create_if_missing(true)
            .filename(&tmp),
    );

    let sync_result = sync::sync(pool, None).await;

    match sync_result {
        Ok(rows_inserted) => {
            // if sync succeeded replace previous database with fresh database + update cache

            // destiny_fetch.db.tmp --> destiny_fetch.db
            fs::rename(tmp, &db)?;

            if let Some(p) = backup {
                // remove old database
                fs::remove_file(p)?;
            }

            // update cache
            cache.data.last_sync_at = Some(chrono::Utc::now());
            cache.data.last_sync_rows_written = Some(rows_inserted);
            cache.write_cache()?;
        }
        Err(e) => {
            // if sync failed restore backup database
            dbg!(e);
            if let Some(p) = backup {
                // destiny_fetch.db.bak --> destiny_fetch.db
                fs::rename(p, db)?;
            }
            // remove failed database
            let _ = fs::remove_file(tmp);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {

    use super::*;

    #[tokio::test]
    async fn test_sync() {
        let cli =
            cli::CLI::try_parse_from(["destiny_fetch", "sync"]).expect("unable to parse command");
        assert!(matches!(cli.cmd, cli::Command::Sync));
        let mut test_cache = Cache::new().expect("failed to create cache");
        let test_db = PathBuf::from("test-data/test.db");
        sync_destinypedia(&mut test_cache, test_db)
            .await
            .expect("sync failed");
        assert!(&test_cache.data.last_sync_at.is_some());
        assert!(&test_cache.data.last_sync_rows_written.is_some());
        let path = test_cache
            .remove_cache()
            .expect("failed to remove test cache");
        println!("Removed test cache at {}", path.display());
    }

    #[tokio::test]
    async fn test_search_simple() {
        let cli_ = cli::CLI::try_parse_from([
            "destiny_fetch",
            "search",
            "-I",
            "exotic",
            "--output",
            "test-data/test_search.txt",
        ])
        .expect("unable to parse search command");
        dbg!(&cli_);
        // assert!(matches!(cli_.cmd, cli::Command::Search(_)));
        assert!(matches!(
            cli_.cmd,
            cli::Command::Search(cli::SearchArgs {
                output: Some(_),
                ..
            })
        ));
        let test_db = PathBuf::from("test-data/STATIC_TEST.db");
        if let cli::Command::Search(args) = cli_.cmd {
            let conn = sqlx::SqlitePool::connect_with(
                SqliteConnectOptions::default()
                    .foreign_keys(true)
                    .filename(&test_db),
            )
            .await
            .expect("unable to create connection pool for test db");
            search::search(&args, conn).await.expect("search failed");
        } else {
            panic!("command could not be parsed as search args: {:?}", cli_);
        }
        let _ = fs::remove_file(Path::new("test-data/test_search.txt"));
    }
}
