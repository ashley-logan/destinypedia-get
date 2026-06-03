use crate::bin_modules::{DestinyFetchError, Result};
use inquire::{Confirm, CustomType, Select, Text, error::InquireError};
use std::path;
use std::{
    fs::File,
    io::{BufRead, BufReader},
};

fn accept_interactive_input(input_type: &str) -> Result<()> {
    let mut count: u8 = 1;
    match input_type {
        "Titles" => {
            let mut titles: Vec<String> = vec![];
            loop {
                let title: String =
                    Text::new(format!("Title {}: (blank input to stop)", count).as_str())
                        .with_default("-1")
                        .prompt()?;

                if title.as_str() == "-1" {
                    break;
                }
                titles.push(title.trim().into());
                count += 1;
            }
        }
        "Ids" => {
            let mut ids: Vec<i32> = vec![];
            loop {
                let id: i32 = CustomType::<i32>::new(
                    format!("Id {}: (blank or negative input to stop)", count).as_str(),
                )
                .with_default(-1_i32)
                .prompt()?;

                if id < 0 {
                    break;
                }
                ids.push(id);
                count += 1;
            }
        }
        _ => (),
    }
    Ok(())
}

fn parse_input_file(input_type: &str) -> Result<()> {
    let mut path = Text::new("Enter input filename: ").prompt()?;
    let mut f = File::open(&path);
    while let Err(_) = f {
        println!("Failed to open filepath {}", &path);
        let again =
            Confirm::new("Would you like to try entering a valid filepath again?").prompt()?;
        if !again {
            return Ok(());
        }
        path = Text::new("Enter input filename: ").prompt()?;
        f = File::open(&path);
    }

    let rdr = BufReader::new(f?);
    match input_type {
        "Titles" => {
            let mut titles: Vec<String> = vec![];
            for r in rdr.lines() {
                titles.push(r?.as_str().trim().into());
            }
        }
        "Ids" => {
            let mut ids: Vec<i32> = vec![];
            for r in rdr.lines() {
                ids.push(r?.parse()?);
            }
        }
        _ => panic!(
            "input type should be either Titles or Ids. got {}",
            &input_type
        ),
    }
    Ok(())
}

pub fn download_prompt() -> Result<()> {
    let opt1: &str =
        Select::new("Download images by ", vec!["Titles", "Ids", "In Category"]).prompt()?;
    match opt1 {
        "Titles" | "Ids" => {
            let opt2 = Select::new(
                format!("How would you like to input image {}?", opt1.to_lowercase()).as_str(),
                vec!["Input File", "Right Here (cli)"],
            )
            .prompt()?;
            match opt2 {
                "Input File" => parse_input_file(opt1),
                "Right Here (cli)" => accept_interactive_input(opt1),
                _ => {}
            }
        }
        "In Category" => (),
        _ => (),
    }
    Ok(())
}
