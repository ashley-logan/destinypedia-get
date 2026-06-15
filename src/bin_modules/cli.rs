use super::{DestinyFetchError, Result};
use crate::bin_modules::database::rows::Ext;
use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, ParseError, Utc};
use clap::{
    Args, Parser, Subcommand, ValueEnum,
    builder::{PathBufValueParser, TypedValueParser},
};
use std::{ops::Mul, path::PathBuf};

// helper function for path directory validation
fn parse_as_dir(p: PathBuf) -> Result<PathBuf> {
    if p.is_dir() {
        Ok(p)
    } else {
        Err(DestinyFetchError::InvalidPathErr)
    }
}

// helper function for path file validation
fn parse_as_file(p: PathBuf) -> Result<PathBuf> {
    if p.is_file() {
        Ok(p)
    } else {
        Err(DestinyFetchError::InvalidPathErr)
    }
}

fn parse_as_utc(s: &str) -> Result<NaiveDateTime> {
    if let Ok(dt) = NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M") {
        Ok(dt)
    } else {
        match NaiveDate::parse_from_str(s, "%Y-%m-%d") {
            Ok(dt) => Ok(dt.and_time(NaiveTime::from_num_seconds_from_midnight_opt(0, 0).unwrap())),
            Err(e) => Err(DestinyFetchError::InvalidTimestampErr(e)),
        }
    }
}

#[derive(Debug, Parser, PartialEq, Eq)]
#[command(version, about = "CLI tool for fetching images from destinypedia.com", long_about = None)]
pub struct CLI {
    #[command(subcommand)]
    cmd: Command, // destinypedia-get [search | download] ...
}

#[derive(Debug, Subcommand, PartialEq, Eq)]
enum Command {
    Search(SearchArgs),
    Download(DownloadArgs),
}

#[derive(Debug, Args, PartialEq, Eq)]
pub struct SearchArgs {
    pub search: String,
    #[command(flatten)]
    pub result_type: ResultType, // filter by images, categories, or both
    #[arg(long = "in-category", short = 'C')]
    pub in_category: Option<String>, // only show results in this category
    #[arg(long, short = 'o')]
    pub output: Option<PathBuf>, // --output [-o] batch1.json
    #[arg(long, short = 'n')]
    pub limit: Option<i32>, // show this many results; default all
    #[command(flatten)]
    pub detail_level: Option<DetailLevel>, // amount of extra information provided for each result
    #[arg(long, value_enum)]
    pub ftype: Option<Vec<Ext>>, // only show images with these filetypes, default all
    #[arg(long)]
    pub maxsize: Option<i32>,
    #[arg(long)]
    pub minsize: Option<i32>,
    #[arg(long, value_parser = parse_as_utc)]
    pub before: Option<NaiveDateTime>,
    #[arg(long, value_parser = parse_as_utc)]
    pub after: Option<NaiveDateTime>,
    #[arg(long)]
    pub maxwidth: Option<i32>,
    #[arg(long)]
    pub minwidth: Option<i32>,
    #[arg(long)]
    pub maxheight: Option<i32>,
    #[arg(long)]
    pub minheight: Option<i32>,
    #[arg(long)]
    pub maxpixels: Option<i32>,
    #[arg(long)]
    pub minpixels: Option<i32>,
}

#[derive(Debug, Args, PartialEq, Eq, Default)]
pub struct DownloadArgs {
    #[command(flatten)]
    pub input: Option<StdinInput>,
    #[command(flatten)]
    pub input_file: Option<FileInput>,

    #[arg(
        long = "output-dir",
        short = 'o',
        help = "Specifies the directory which images will be downloaded to. This directory must already exist (default = $CWD)"
    )]
    #[arg(value_parser = PathBufValueParser::new().try_map(parse_as_dir))]
    pub output_dir: Option<PathBuf>, // --target-dir [-d] /media/d2/

    #[arg(
        long,
        help = "Don't show any confirmation prompts, forces this command to run uninteractively"
    )]
    pub noconfirm: bool,
}

impl DownloadArgs {
    pub fn no_input(&self) -> bool {
        match self {
            DownloadArgs {
                input: None,
                input_file: None,
                ..
            } => true,
            _ => false,
        }
    }
}

#[derive(Args, Debug, Clone, PartialEq, Eq, Default)]
#[group(required = false, multiple = true, conflicts_with = "StdinInput")]
pub struct FileInput {
    #[arg(long = "titles-input", value_parser = PathBufValueParser::new().try_map(parse_as_file), help = "Path to file of line delimited image titles")]
    pub titles_input: Option<PathBuf>,

    #[arg(long = "ids-input", value_parser = PathBufValueParser::new().try_map(parse_as_file), help = "Path to file of line delimited image ids")]
    pub ids_input: Option<PathBuf>,

    #[arg(long = "from-cached", help = "Name of a previously cached search")]
    pub from_cached: Option<String>,
}

#[derive(Args, Debug, Clone, PartialEq, Eq, Default)]
#[group(required = false, multiple = true)]
pub struct StdinInput {
    #[arg(
        long,
        num_args = 1..,
        help = "Space sparated image titles, each title should be surrouned by quotes"
    )]
    pub titles: Option<Vec<String>>,

    #[arg(long, num_args = 1.., help = "Space separated images ids, all ids must be positive")]
    pub ids: Option<Vec<u16>>,

    #[arg(
        long = "in-category",
        help = "Will target all images within this category"
    )]
    pub in_category: Option<String>,
}

#[derive(Args, Debug, Clone, PartialEq, Eq)]
#[group(required = true, multiple = false)]
pub struct ResultType {
    /// return image results only
    #[arg(long, short = 'I')]
    pub images: bool,

    /// return category results only
    #[arg(long, short = 'C')]
    pub categories: bool,

    /// returns both category and image results
    #[arg(long, short = 'A')]
    pub all: bool,
}

#[derive(Args, Debug, Clone, PartialEq, Eq)]
#[group(required = false, multiple = false)]
pub struct DetailLevel {
    /// output all category/images information
    #[arg(long, short = 'd')]
    pub detailed: bool,

    /// output onlu category/image titles
    #[arg(long)]
    pub titles: bool,

    /// output only category/image ids
    #[arg(long)]
    pub ids: bool,

    /// output important category/image information (default)
    #[arg(long)]
    pub default: bool,
}

#[derive(Debug, Args, PartialEq, Eq)]
#[group(required = true, multiple = false)]
pub struct Pattern {
    #[arg(long, help = "download all images from all specified pages")]
    all: bool,

    #[arg(long, value_name = "[img1, img2, ...]")]
    images: Vec<String>,

    #[arg(
        long = "images-input",
        value_name = "IMAGES_INPUT_FILE",
        value_parser = PathBufValueParser::new().try_map(parse_as_file),
        help = "Line seperated text file that contains targeted image name(s)"
    )]
    images_input: PathBuf, // --images-input /home/meep/img_names.txt
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;
    use clap::{Command, CommandFactory};
    use tempfile::{NamedTempFile, TempDir};

    #[test]
    fn test_command_search1() {
        let r = CLI::command()
            .try_get_matches_from([
                "destiny_fetch",
                "search",
                "--images",
                "-c",
                "Images of Pulse Rifles",
                "Out",
            ])
            .unwrap();
        assert_eq!(r.subcommand_name(), Some("search"));
        let sub = r
            .subcommand_matches("search")
            .expect("no matches for search");
        assert!(sub.get_one::<bool>("images").is_some());
        assert_eq!(
            sub.get_one("in_category"),
            Some(&String::from("Images of Pulse Rifles"))
        );
    }

    #[test]
    fn test_command_search2() {
        let cli = CLI::try_parse_from([
            "destiny_fetch",
            "search",
            "Hive",
            "--all",
            "--in-category",
            "Images of Hive",
            "--minwidth",
            "1080",
            "--minheight",
            "920",
            "--maxsize",
            "5000",
            "--ftype",
            "png",
            "--ftype",
            "jpg",
            "--before",
            "2020-12-25",
        ])
        .expect("unable to parse args");
        let exp = CLI {
            cmd: super::Command::Search(SearchArgs {
                search: "Hive".into(),
                result_type: ResultType {
                    images: false,
                    categories: false,
                    all: true,
                },
                in_category: Some("Images of Hive".into()),
                output: None,
                limit: None,
                detail_level: None,
                ftype: Some(vec![Ext::PNG, Ext::JPG]),
                maxsize: Some(5000),
                minsize: None,
                before: NaiveDateTime::parse_from_str("2020-12-25 00:00", "%Y-%m-%d %H:%M").ok(),
                after: None,
                maxwidth: None,
                minwidth: Some(1080),
                maxheight: None,
                minheight: Some(920),
                maxpixels: None,
                minpixels: None,
            }),
        };
        assert_eq!(cli, exp);
    }

    #[test]
    fn test_parse_dir_success() {
        let dir = TempDir::new().unwrap();
        let pbuf: PathBuf = PathBuf::from(dir.path());

        assert!(parse_as_dir(pbuf).is_ok())
    }

    #[test]
    fn test_parse_dir_fail() {
        let not_dir = NamedTempFile::new().unwrap();
        let pbuf: PathBuf = PathBuf::from(not_dir.path());

        assert!(parse_as_dir(pbuf).is_err())
    }

    #[test]
    fn test_parse_file_success() {
        let f = NamedTempFile::new().unwrap();
        let pbuf: PathBuf = PathBuf::from(f.path());

        assert!(parse_as_file(pbuf).is_ok())
    }

    #[test]
    fn test_parse_file_fail_1() {
        let not_file: TempDir = TempDir::new().unwrap();
        let pbuf: PathBuf = PathBuf::from(not_file.path());

        assert!(parse_as_file(pbuf).is_err())
    }

    #[test]
    fn test_parse_file_fail_2() {
        let f = NamedTempFile::new().unwrap();
        let mut pbuf = PathBuf::from(f.path());
        pbuf.push("non-existent-file");

        assert!(parse_as_file(pbuf).is_err())
    }
}
