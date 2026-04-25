/*
   cat api.php\?action\=query\&titles\=File\:Dire_Taken_Concept_1.jpg\&prop\=imageinfo\&iiprop\=url\&format\=json
{"batchcomplete":"","query":{"normalized":[{"from":"File:Dire_Taken_Concept_1.jpg","to":"File:Dire Taken Concept 1.jpg"}],
"pages":{"50985":{"pageid":50985,"ns":6,"title":"File:Dire Taken Concept 1.jpg","imagerepository":"local",
"imageinfo":[{"url":"https://destiny.wiki.gallery/images/9/96/Dire_Taken_Concept_1.jpg","descriptionurl":"https://www.destinypedia.com/File:Dire_Taken_Concept_1.jpg","descriptionshorturl":"https://www.destinypedia.com/index.php?curid=50985"}]}}}}
*/
use clap::{
    Args, Parser, Subcommand,
    builder::{PathBufValueParser, TypedValueParser},
};
use std::path::PathBuf;

fn parse_as_dir(p: PathBuf) -> Result<PathBuf, &'static str> {
    if p.is_dir() {
        Ok(p)
    } else {
        Err("target download path must be an intialized directory")
    }
}

fn parse_as_file(p: PathBuf) -> Result<PathBuf, &'static str> {
    if p.is_file() {
        Ok(p)
    } else {
        Err("Input file must exist")
    }
}

// fn parse_path<F: Fn(&PathBuf) -> bool>(p: PathBuf, cond: F, err_msg: &'static str) -> Result<PathBuf, &'static str>{
//     if cond(&p) {
//         Ok(p)
//     } else {
//         Err(err_msg)
//     }
// }


#[derive(Parser)]
#[command(version, about = "CLI tool for fetching images from destinypedia.com", long_about = None)]
pub struct CLI {
    #[command(subcommand)]
    cmd: Commands, // destinypedia-get [fetch | download] ...

    #[command(flatten)]
    page_input: Pages
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

    #[arg(long = "images-input", value_name = "IMAGES_INPUT_FILE", value_parser = PathBufValueParser::new().try_map(parse_as_file), help = "Line seperated text file that contains targeted image name(s)")]
    images_input: PathBuf // --images-input /home/meep/img_names.txt
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
        help = "Line seperated text file that contains page name(s)")]
    page_file: PathBuf, // --input-file [-i] /home/meep/target_pages.txt
}

