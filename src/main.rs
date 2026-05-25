pub mod bin_modules;
use bin_modules::{DestinyFetchError, Result, cli, database, get, sync};
use clap::Parser;
use dirs;
use std::{fs, path};

fn main() {
    let _cli: cli::CLI = cli::CLI::parse();
}

async fn sync_destinypedia() -> Result<()> {
    let db = dirs::data_local_dir()
        .or(dirs::data_dir())
        .ok_or(DestinyFetchError::IOErr)?
        .join("destiny_fetch.db");
    let tmp = db.with_added_extension("tmp");

    let backup: Option<path::PathBuf> = {
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
