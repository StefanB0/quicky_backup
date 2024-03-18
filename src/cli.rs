use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// performs backup of target directory
    Backup {
        /// The target location where the backup will be stored
        #[arg(short, long)]
        target: PathBuf,

        /// The directories and files that will be backed up
        #[arg(value_name = "DIR/FILE")]
        files: Vec<PathBuf>
    },
    /// performs recovery of target backup
    Restore {
        /// The target location where the backup is
        #[arg(short, long)]
        target: PathBuf,

        /// The directory where the backup will be restored
        #[arg(value_name = "DIR")]
        files: Option<PathBuf>
    },
}