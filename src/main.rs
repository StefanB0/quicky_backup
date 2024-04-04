
use clap::Parser;

mod cli;
mod backup_vault;

use cli::Cli;


fn main() {
    let cli = Cli::parse();
    cli.execute();
}