use clap::Parser;

mod cli;
mod error;
mod get;

use cli::*;
fn main() {
    let _cli: CLI = CLI::parse();
}
