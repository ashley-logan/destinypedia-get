pub mod bin_modules;
use bin_modules::cli::Command;
use bin_modules::{Cache, DestinyFetchError, Result, cli, database, download, search, sync};
use chrono::{DateTime, TimeDelta, Utc};
use clap::Parser;
use dirs;
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::{Sqlite, SqlitePool};
use std::{fs, path::PathBuf};

pub static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!();

#[tokio::main]
async fn main() {
    let _cli: cli::CLI = cli::CLI::parse();
    let cache: Cache = match Cache::open_or_create(None) {
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

    let mut needs_sync = match cache.last_sync_at {
        Some(dt) => (Utc::now() - TimeDelta::weeks(2) >= dt) | !db_path.exists(),
        None => true,
    };

    match _cli.cmd {
        Command::Search(args) => {
            if needs_sync {
                match sync_destinypedia().await {
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
            let pool = SqlitePool::connect_lazy_with(
                SqliteConnectOptions::default()
                    .foreign_keys(false)
                    .filename(&db_path),
            );
            let result = search::search(args, &pool).await;
            match result {
                Ok(()) => {
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
            }
        }
        Command::Download(args) => {
            if needs_sync {
                match sync_destinypedia().await {
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
            let pool = SqlitePool::connect_lazy_with(
                SqliteConnectOptions::default()
                    .foreign_keys(false)
                    .filename(&db_path),
            );
            let result = download::download(args, pool).await;
            match result {
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
            }
        }
        Command::Sync => {
            let result = sync_destinypedia().await;
            match result {
                Ok(()) => {
                    println!("Database sync successful!")
                }
                Err(e) => {
                    println!(
                        "ERROR: Failed to sync database with error: {}",
                        e.to_string()
                    );
                }
            }
        }
    }
}

async fn sync_destinypedia() -> Result<()> {
    let db = dirs::data_local_dir()
        .or(dirs::data_dir())
        .ok_or(DestinyFetchError::InvalidPathErr)?
        .join("destiny_fetch.db");
    let tmp = db.with_added_extension("tmp");

    let backup: Option<PathBuf> = {
        if fs::exists(&db)? {
            sync::create_backup(&db).ok()
        } else {
            None
        }
    };

    let sync_result = sync::sync(&tmp.to_string_lossy(), None).await;

    match sync_result {
        Ok(map) => {
            fs::rename(tmp, db)?;

            if let Some(p) = backup {
                fs::remove_file(p)?;
            }

            todo!("turn map into cache payload")
        }
        Err(e) => {
            dbg!(e);
            if let Some(p) = backup {
                fs::rename(p, db)?;
            }
            let _ = fs::remove_file(tmp);
        }
    }

    Ok(())
}
