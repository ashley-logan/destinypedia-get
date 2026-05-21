pub mod bin_modules;
use bin_modules::{DestinyFetchError, Result, cli, database, get, sync};
use clap::Parser;
use dirs;
use std::{fs, path};

fn main() {
    let _cli: cli::CLI = cli::CLI::parse();
}

async fn sync_destinypedia() -> Result<()> {
    let db = dirs::data_dir()
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

    sync::sync(&tmp, None).await?;

    fs::rename(tmp, db)?;

    if let Some(p) = backup {
        fs::remove_file(p)?;
    }

    Ok(())
}
