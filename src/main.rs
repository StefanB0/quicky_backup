use std::fs;
use std::path::PathBuf;

use clap::{CommandFactory, Parser, Subcommand};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
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

fn naive_copy_file(input: &PathBuf, output_dir: &PathBuf) -> std::io::Result<()> {
    // Create the output directory if it doesn't exist
    fs::create_dir_all(&output_dir)?;
    
    let file_name = input
        .file_name()
        .unwrap_or_default()
        .to_str()
        .unwrap_or("");

    let file_path = input.to_str().unwrap_or_default();
    
    let dest_path = output_dir.join(file_name);

    fs::copy(&file_path, &dest_path)?;

    Ok(())
}

fn naive_copy_dir(input_dir: &PathBuf, output_dir: &PathBuf) -> std::io::Result<()> {
    // Create the output directory if it doesn't exist
    fs::create_dir_all(&output_dir)?;

    for entry in fs::read_dir(&input_dir)? {
        let entry = entry?;

        if entry.file_type()?.is_dir() {
            let dir_name = entry.file_name();
            let dir_path = entry.path();

            let dest_path = output_dir.join(dir_name);

            naive_copy_dir(&dir_path, &dest_path)?;
        } else if entry.file_type()?.is_file() {
            let file_name = entry.file_name();
            let file_path = entry.path();

            let dest_path = output_dir.join(file_name);

            fs::copy(&file_path, &dest_path)?;
        }
    }

    Ok(())
}

fn is_directory_empty(path: &PathBuf) -> bool {
    match fs::read_dir(path) {
        Ok(mut dir) => dir.next().is_none(),
        Err(_) => true
    }
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Backup { target, files }) => {
            println!("The target location is \"{0}\"", target.to_str().expect(""));

            fs::create_dir_all(&target).unwrap();
            if !is_directory_empty(&target) {
                println!("The target location is not empty. Do you want to continue? [y/N]");
                let mut response = String::new();
                std::io::stdin().read_line(&mut response).unwrap();
                if response.to_lowercase().trim() != "y" {
                    println!("Aborting...");
                    std::process::exit(1);
                }
            }

            for entry in files {
                if entry.is_dir() {
                    naive_copy_dir(&entry, &target).unwrap()
                } else if entry.is_file() {
                    naive_copy_file(&entry, &target).unwrap()
                }
            }
        },
        Some(Commands::Restore { target, files: _ }) => {
            println!("The target location is |{0}|", target.to_str().expect(""));
        },
        None => {
            Cli::command().print_help().err();
        }
    }
}