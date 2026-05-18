pub mod bin_modules;
use bin_modules::{cli, database, get, sync};
use clap::Parser;
fn main() {
    let _cli: cli::CLI = cli::CLI::parse();
}
