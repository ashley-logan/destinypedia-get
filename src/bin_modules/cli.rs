use super::{DestinyFetchError, Result};
use chrono::{DateTime, Utc};
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

fn parse_as_utc(s: &str) -> Result<DateTime<Utc>> {
    if let Ok(t3) = DateTime::parse_from_rfc3339(s) {
        Ok(t3.into())
    } else if let Ok(t2) = DateTime::parse_from_rfc2822(s) {
        Ok(t2.into())
    } else {
        match DateTime::parse_from_str(s, "%Y-%m-%d") {
            Ok(t1) => Ok(t1.into()),
            Err(e) => Err(e)?,
        }
    }
}

fn parse_search_command(args: SearchArgs) {
    ()
}

fn parse_download_command(args: DownloadArgs) {
    ()
}

pub fn handle_args(cli: CLI) {
    match cli.cmd {
        Command::Search(s) => parse_search_command(s),
        Command::Download(d) => parse_download_command(d),
    }
}

#[derive(Parser)]
#[command(version, about = "CLI tool for fetching images from destinypedia.com", long_about = None)]
pub struct CLI {
    #[command(subcommand)]
    cmd: Command, // destinypedia-get [search | download] ...
}

#[derive(Debug, Subcommand)]
enum Command {
    Search(SearchArgs),
    Download(DownloadArgs),
}

#[derive(Debug, Args)]
pub struct SearchArgs {
    pub search: String,
    #[command(flatten)]
    pub result_type: ResultType, // filter by images, categories, or both
    #[arg(long = "in-category", short = 'c')]
    pub in_category: Option<String>, // only show results in this category
    #[arg(long, short = 'o')]
    pub output: Option<PathBuf>, // --output [-o] batch1.json
    #[arg(long, short = 'n')]
    pub limit: Option<i32>, // show this many results; default all
    #[command(flatten)]
    pub detail_level: Option<DetailLevel>, // amount of extra information provided for each result
    #[arg(long, value_enum)]
    pub ftype: Option<Vec<FileType>>, // only show images with these filetypes, default all
    #[arg(long)]
    pub maxsize: Option<i32>,
    #[arg(long)]
    pub minsize: Option<i32>,
    #[arg(long, value_parser = parse_as_utc)]
    pub before: Option<DateTime<Utc>>,
    #[arg(long, value_parser = parse_as_utc)]
    pub after: Option<DateTime<Utc>>,
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

// authors::table
//     .inner_join(author_books::table.on(author_books::author_id.eq(authors::id)))
//     .inner_join(books::table.on(author_books::book_id.eq(books::id)))

// let page_with_book = pages::table
//     .inner_join(books::table)
//     .filter(books::title.eq("Momo"))
//     .select((Page::as_select(), Book::as_select()))
//     .load::<(Page, Book)>(conn)?;

#[derive(Debug, Args)]
pub struct DownloadArgs {
    #[command(flatten)]
    pattern: Pattern, // [--all | --images MaraSovConceptArt1.jpg "Thorn Wishes of Sorrow.jpg" Destiny_TK_Golgoroth\'s_Cellar.jpg]

    #[arg(
        long = "target-dir",
        short = 'd',
        value_name = "DOWNLOAD_DIR",
        help = "Specifies the directory which images will be downloaded to. This directory must already exist (default = $CWD)"
    )]
    #[arg(value_parser = PathBufValueParser::new().try_map(parse_as_dir))]
    target_dir: Option<PathBuf>, // --target-dir [-d] /media/d2/
}

#[derive(Subcommand)]
pub(crate) enum Commands {
    Search {
        #[command(flatten)]
        result_type: ResultType, // filter by images, categories, or both
        #[arg(long = "in-category", short = 'c')]
        in_category: Option<String>, // only show results in this category
        #[arg(long, short = 'o')]
        output: Option<PathBuf>, // --output [-o] batch1.json
        #[arg(long, short = 'n')]
        limit: Option<i32>, // show this many results; default all
        #[command(flatten)]
        detail_level: Option<DetailLevel>, // amount of extra information provided for each result
        #[arg(long, value_enum)]
        ftype: Option<Vec<FileType>>, // only show images with these filetypes, default all
        #[arg(long)]
        maxsize: Option<i32>,
        #[arg(long)]
        minsize: Option<i32>,
        #[arg(long)]
        before: Option<String>,
        #[arg(long)]
        after: Option<String>,
        #[arg(long)]
        maxwidth: Option<i32>,
        #[arg(long)]
        minwidth: Option<i32>,
        #[arg(long)]
        maxheight: Option<i32>,
        #[arg(long)]
        minheight: Option<i32>,
        #[arg(long)]
        maxpixels: Option<i32>,
        #[arg(long)]
        minpixels: Option<i32>,
    },
    Download {
        #[command(flatten)]
        pattern: Pattern, // [--all | --images MaraSovConceptArt1.jpg "Thorn Wishes of Sorrow.jpg" Destiny_TK_Golgoroth\'s_Cellar.jpg]

        #[arg(
            long = "target-dir",
            short = 'd',
            value_name = "DOWNLOAD_DIR",
            help = "Specifies the directory which images will be downloaded to. This directory must already exist (default = $CWD)"
        )]
        #[arg(value_parser = PathBufValueParser::new().try_map(parse_as_dir))]
        target_dir: Option<PathBuf>, // --target-dir [-d] /media/d2/
    },
}

#[derive(Args, Debug, Clone)]
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

#[derive(Args, Debug, Clone)]
#[group(required = false, multiple = false)]
pub(crate) struct DetailLevel {
    /// return image results only
    #[arg(long, short = 'd')]
    detailed: bool,

    /// return category results only
    #[arg(long, short = 's')]
    simple: bool,

    /// returns both category and image results
    #[arg(long)]
    default: bool,
}

#[derive(Debug, Clone, ValueEnum)]
pub(crate) enum FileType {
    PNG,
    JPG,
    WEBP,
    GIF,
    HEIC,
    SVG,
}

#[derive(Debug, Args)]
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

#[derive(Debug, Args)]
#[group(required = true, multiple = false)]
pub struct Pages {
    #[arg(
        value_name = "PAGES",
        help = "Page names must be seperated by a space. The name itself must replace all internal spaces as underscores."
    )]
    pages: Vec<String>, // ... Taken Ascendant_Plane_(location) Mara_Sov

    #[arg(
        value_name = "PAGES_INPUT_FILE",
        long = "input-file",
        short = 'i',
        value_parser = PathBufValueParser::new().try_map(parse_as_file),
        help = "Line seperated text file that contains page name(s)"
    )]
    page_file: PathBuf, // --input-file [-i] /home/meep/target_pages.txt
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::{NamedTempFile, TempDir};

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
