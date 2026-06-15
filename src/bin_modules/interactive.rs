use super::cache::*;
use super::cli::{DetailLevel, DownloadArgs, FileInput, ResultType, StdinInput};
use super::download::{validate_category, validate_ids, validate_titles};
use super::input::*;
use crate::bin_modules::{
    DestinyFetchError, Result,
    database::rows::{CategoriesRow, ImagesRow},
};
use inquire::{Confirm, CustomType, Select, Text, error::InquireError};
use sqlx::Connection;
use sqlx::sqlite::{SqlitePool, SqliteRow};
use sqlx::{Pool, QueryBuilder, Sqlite, query_builder::Separated};
use std::collections::HashSet;
use std::path;
use std::str::FromStr;
use std::{
    fs::File,
    io::{BufRead, BufReader},
};

fn prompt_titles() -> Result<Vec<String>> {
    let mut count: u16 = 1;
    let mut titles: HashSet<String> = HashSet::new();
    loop {
        let title: String = Text::new(format!("Title {}: (blank input to finish)", count).as_str())
            .with_default("-1")
            .prompt()?;

        if title.as_str() == "-1" {
            break;
        }
        let dup = !titles.insert(title.trim().into());
        match dup {
            true => println!(
                "Title: {} was already entered. Enter a new title or blank to finish input.",
                title
            ),
            false => count += 1,
        }
    }
    println!("Validating {} image titles", count - 1);
    Ok(titles.into_iter().collect())
}

fn prompt_ids() -> Result<Vec<u16>> {
    let mut count: u16 = 1;
    let mut ids: HashSet<u16> = HashSet::new();
    loop {
        let id: i32 = CustomType::<i32>::new(
            format!("Id {}: (blank or negative input to finish) ", count).as_str(),
        )
        .with_default(-1_i32)
        .prompt()?;

        if id < 0 {
            break;
        }
        let dup = !ids.insert(
            id.try_into()
                .map_err(|_| DestinyFetchError::NegativeArgErr)?,
        );
        match dup {
            true => println!(
                "Id: {} was already entered. Enter a new title or blank to finish input.",
                id
            ),
            false => count += 1,
        }
    }
    println!("Validating {} image ids", count - 1);
    Ok(ids.into_iter().collect())
}

fn prompt_input_file() -> Result<path::PathBuf> {
    let mut s: String = Text::new("Enter path to file (file must contain one value per line): ")
        .with_default("-1")
        .prompt()?;

    if &s[..] == "-1" {
        return Err(DestinyFetchError::Quit);
    }
    let mut p = path::Path::new(s.trim());
    while !p.is_file() {
        println!("Path {} does not point to an existing file", p.display());
        match Confirm::new("Would you like to try entering a valid filepath again?").prompt() {
            Ok(true) => {
                s = Text::new("Enter path to file (file must contain one value per line): ")
                    .with_default("-1")
                    .prompt()?;
                if &s[..] == "-1" {
                    return Err(DestinyFetchError::Quit);
                }
                p = path::Path::new(s.trim());
            }
            Ok(false)
            | Err(InquireError::OperationCanceled | InquireError::OperationInterrupted) => {
                return Err(DestinyFetchError::Quit);
            }
            Err(e) => return Err(e)?,
        }
    }
    Ok(p.to_path_buf())
}

pub fn prompt_output_dir() -> Result<path::PathBuf> {
    let mut s: String = Text::new("Enter path to output directory (.) ")
        .with_default(".")
        .prompt()?;
    let mut p: path::PathBuf = match &s[..] {
        "." => std::env::current_dir()
            .map_err(|_| DestinyFetchError::InvalidPathErr)?
            .to_path_buf(),
        _p => _p.into(),
    };
    while !p.is_dir() {
        println!(
            "Path {} does not point to an existing directory",
            p.display()
        );
        match Confirm::new("Would you like to try entering a valid directory path again?").prompt()
        {
            Ok(true) => {
                s = Text::new("Enter path to output directory (.) ")
                    .with_default(".")
                    .prompt()?;
                p = match &s[..] {
                    "." => std::env::current_dir()
                        .map_err(|_| DestinyFetchError::InvalidPathErr)?
                        .to_path_buf(),
                    _p => _p.into(),
                };
            }
            Ok(false)
            | Err(InquireError::OperationCanceled | InquireError::OperationInterrupted) => {
                return Err(DestinyFetchError::Quit);
            }
            Err(e) => return Err(e)?,
        }
    }
    Ok(p)
}

pub fn promp_download_input() -> Result<DownloadArgs> {
    let mut args: DownloadArgs = DownloadArgs::default();
    let opt1: &str =
        Select::new("Download images by ", vec!["Titles", "Ids", "In Category"]).prompt()?;
    match opt1 {
        "Titles" => {
            let opt2 = Select::new(
                "How would you like to input image titles?",
                vec!["Right Here (cli)", "Input File"],
            )
            .prompt()?;
            let titles = match opt2 {
                "Input File" => titles_from_file(prompt_input_file()?)?,
                "Right Here (cli)" => prompt_titles()?,
                _ => return Err(DestinyFetchError::Unknown),
            };
            let input: StdinInput = StdinInput {
                ids: None,
                titles: Some(titles),
                in_category: None,
            };
            args.input = Some(input);
        }
        "Ids" => {
            let opt2 = Select::new(
                "How would you like to input image ids?",
                vec!["Right Here (cli)", "Input File"],
            )
            .prompt()?;
            let ids = match opt2 {
                "Input File" => ids_from_file(prompt_input_file()?)?,
                "Right Here (cli)" => prompt_ids()?,
                _ => return Err(DestinyFetchError::Unknown),
            };
            let input: StdinInput = StdinInput {
                ids: Some(ids),
                titles: None,
                in_category: None,
            };
            args.input = Some(input);
        }
        "In Category" => {
            let cat = Text::new("Enter a valid category name: ")
                .with_default("-1")
                .prompt()?;
            if &cat == "-1" {
                return Err(DestinyFetchError::Quit);
            }
            let input: StdinInput = StdinInput {
                ids: None,
                titles: None,
                in_category: Some(cat),
            };
            args.input = Some(input);
        }
        _ => return Err(DestinyFetchError::Unknown),
    }

    Ok(args)
}

pub fn prompt_confirm_download(images: &[ImagesRow], path: &path::PathBuf) -> bool {
    println!("Images to be downloaded:");
    let mut total_size: u64 = 0;
    for img in images.iter() {
        total_size += img.size.try_into().unwrap_or(0);
        println!(
            "\t({}), {}, {}x{}, {}KiB",
            img.id, img.title, img.width, img.height, img.size
        );
    }
    if total_size == 0 {
        println!("ERROR: No images prepared for download...ending program");
        return false;
    }
    println!(
        "{} images to be downloaded ({}MiB) to folder {}",
        images.len(),
        total_size / 1024,
        path.display()
    );
    match Confirm::new("Would you like to continue?").prompt() {
        Ok(b) => b,
        Err(_) => false,
    }
}

pub async fn download_prompt(conn: SqlitePool) -> Result<Vec<ImagesRow>> {
    let opt1: &str =
        Select::new("Download images by ", vec!["Titles", "Ids", "In Category"]).prompt()?;
    let images = match opt1 {
        "Titles" => {
            let opt2 = Select::new(
                "How would you like to input image titles?",
                vec!["Right Here (cli)", "Input File"],
            )
            .prompt()?;
            let titles = match opt2 {
                "Input File" => parse_titles_file(prompt_input_file()?).await?,
                "Right Here (cli)" => prompt_titles()?.into_iter().collect(),
                _ => return Err(DestinyFetchError::Unknown),
            };
            validate_titles(titles, conn).await
        }
        "Ids" => {
            let opt2 = Select::new(
                "How would you like to input image ids?",
                vec!["Right Here (cli)", "Input File"],
            )
            .prompt()?;
            let ids = match opt2 {
                "Input File" => parse_ids_file(prompt_input_file()?).await?,
                "Right Here (cli)" => prompt_ids()?.into_iter().collect(),
                _ => return Err(DestinyFetchError::Unknown),
            };
            validate_ids(ids, conn).await
        }
        "In Category" => {
            let cat = Text::new("Enter a valid category name: ")
                .with_default("-1")
                .prompt()?;
            if &cat == "-1" {
                return Err(DestinyFetchError::Quit);
            }
            validate_category(cat, conn).await
        }
        _ => Err(DestinyFetchError::Unknown),
    }?;
    println!("Images to be downloaded:");
    let mut total_size: u64 = 0;
    for img in images.iter() {
        total_size += img.size.try_into().unwrap_or(0);
        println!(
            "\t({}), {}, {}x{}, {}KiB",
            img.id, img.title, img.width, img.height, img.size
        );
    }
    if total_size == 0 {
        println!("ERROR: No images prepared for download...ending program");
        return Err(DestinyFetchError::Quit);
    }
    let opt4 = Select::new(
        format!(
            "{} images to be downloaded ({}MiB)\nHow would you like to proceed?",
            images.len(),
            total_size / 1024
        )
        .as_str(),
        vec![
            "Choose target folder for images",
            "Cache download for later and Quit",
            "Quit",
        ],
    )
    .prompt()?;
    match opt4 {
        "Choose target folder for images" => {
            let dir = prompt_output_dir()?;
        }
        "Cache download for later and Quit" => {
            let mut cache: Cache = Cache::open()?;
            let default_name: String = cache.new_save_name();
            let cache_name = Text::new(
                format!(
                    "Enter save name or leave blank for default ({})",
                    &default_name
                )
                .as_str(),
            )
            .with_default(&default_name[..])
            .prompt()?;
            cache.store_images(cache_name, images)?;
            println!(
                "Successfully saved image metadata with save name {}",
                &cache_name
            );
            return Err(DestinyFetchError::Quit);
        }
        "Quit" => return Err(DestinyFetchError::Quit),
        _ => return Err(DestinyFetchError::Unknown),
    }
}
