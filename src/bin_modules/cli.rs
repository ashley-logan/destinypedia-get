use clap::{
    Args, Parser, Subcommand,
    builder::{PathBufValueParser, TypedValueParser},
};
use std::path::PathBuf;

// helper function for path directory validation
fn parse_as_dir(p: PathBuf) -> Result<PathBuf, &'static str> {
    if p.is_dir() {
        Ok(p)
    } else {
        Err("target download path must be an intialized directory")
    }
}

// helper function for path file validation
fn parse_as_file(p: PathBuf) -> Result<PathBuf, &'static str> {
    if p.is_file() {
        Ok(p)
    } else {
        Err("Input file must exist")
    }
}

#[derive(Parser)]
#[command(version, about = "CLI tool for fetching images from destinypedia.com", long_about = None)]
pub struct CLI {
    #[command(subcommand)]
    cmd: Commands, // destinypedia-get [fetch | download] ...

    #[command(flatten)]
    page_input: Pages,
}

#[derive(Subcommand)]
pub enum Commands {
    Fetch {
        #[arg(long, short = 'o', value_name = "OUTPUT_FILE")]
        output: Option<PathBuf>, // --output [-o] batch1.json
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
#[derive(Args)]
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

#[derive(Args)]
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
