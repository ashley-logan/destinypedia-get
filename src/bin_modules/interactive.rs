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
use std::{
    fs::File,
    io::{BufRead, BufReader},
};

fn prompt_titles() -> Result<HashSet<String>> {
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
    Ok(titles)
}

fn prompt_ids() -> Result<HashSet<i32>> {
    let mut count: u16 = 1;
    let mut ids: HashSet<i32> = HashSet::new();
    loop {
        let id: i32 = CustomType::<i32>::new(
            format!("Id {}: (blank or negative input to finish)", count).as_str(),
        )
        .with_default(-1_i32)
        .prompt()?;

        if id < 0 {
            break;
        }
        let dup = !ids.insert(id);
        match dup {
            true => println!(
                "Id: {} was already entered. Enter a new title or blank to finish input.",
                id
            ),
            false => count += 1,
        }
    }
    println!("Validating {} image ids", count - 1);
    Ok(ids)
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

async fn download_from_titles(db_url: &str) -> Result<()> {
    let pool = SqlitePool::connect_lazy(db_url)?;
    let opt2 = Select::new(
        "How would you like to input image titles?",
        vec!["Right Here (cli)", "Input File"],
    )
    .prompt()?;
    let titles: HashSet<String> = match opt2 {
        "Input File" => parse_titles_file()?,
        "Right Here (cli)" => prompt_titles()?,
        _ => return Err(DestinyFetchError::Unknown),
    };
    let valid_titles: HashSet<String> = validate_titles(&titles, &pool).await?;
    for invalid_title in titles.difference(&valid_titles) {
        println!("Warning: no image found matching title {}", invalid_title);
    }
    let mut q: QueryBuilder<Sqlite> =
        QueryBuilder::new("SELECT * FROM images WHERE LOWER(title) IN ");
    q.push_tuples(valid_titles.iter(), |mut b, title| {
        b.push_bind(title);
    });
    let rows: Vec<ImagesRow> = q.build_query_as().fetch_all(&pool).await?;
    println!("Images to be downloaded:");
    let mut total_size: u64 = 0;
    for img in rows.iter() {
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
    let options = vec![
        "Choose target folder for images",
        "Cache download for later and Quit",
        "Quit",
    ];
    let opt4 = Select::new(
        format!(
            "{} images to be downloaded ({}MiB)\nHow would you like to proceed?",
            rows.len(),
            total_size / 1024
        )
        .as_str(),
        options,
    )
    .prompt()?;
    match opt4 {
        "Choose target folder for images" => (),
        "Cache download for later and Quit" => (),
        "Quit" | _ => return Err(DestinyFetchError::Quit),
    }

    Ok(())
}

pub async fn download_prompt(db_url: &str) -> Result<()> {
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
                "Input File" => parse_titles_file()?,
                "Right Here (cli)" => prompt_titles()?,
                _ => return Err(DestinyFetchError::Unknown),
            };
        }
        "Ids" => {
            let opt2 = Select::new(
                "How would you like to input image ids?",
                vec!["Right Here (cli)", "Input File"],
            )
            .prompt()?;
            let ids = match opt2 {
                "Input File" => parse_ids_file()?,
                "Right Here (cli)" => prompt_ids()?,
                _ => return Err(DestinyFetchError::Unknown),
            };
        }
        "In Category" => (),
        _ => (),
    }
    Ok(())
}
