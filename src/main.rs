use clap::Parser;

mod cli;
mod get;

use cli::*;
fn main() {
    let _cli: CLI = CLI::parse();
}
