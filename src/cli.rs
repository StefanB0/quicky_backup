use std::{fs, io::Write};
use std::path::PathBuf;

use clap::{CommandFactory, Parser, Subcommand};

use crate::backup_vault::BackupVault;
use crate::backup_vault::BackupError;

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
        vault: PathBuf,

        /// The id of the snapshot to be restored
        #[arg(short, long)]
        snapshot: Option<String>,

        /// The directory where the backup will be restored
        #[arg(value_name = "DIR")]
        target: PathBuf,
    },
}

impl Cli {
    pub fn execute(&self) {
        match &self.command {
            Some(Commands::Backup { target, files }) => {
                let password = ask_for_password();

                let mut backup_vault = match BackupVault::open(target, &password) {
                    Ok(vault) => vault,
                    Err(BackupError::VaultWrongPassword) => {
                        println!("Wrong password");
                        std::process::exit(1);
                    },
                    Err(BackupError::VaultDoesNotExist) => match BackupVault::create(target, &password) {
                        Ok(vault) => vault,
                        Err(_) => {
                            println!("Failed to create vault");
                            std::process::exit(1);
                        }
                    },
                    Err(err) => {
                        println!("Failed to open vault");
                        dbg!(err);
                        std::process::exit(1);
                    }
                };

                backup_vault.backup(files).expect("backup-vault failed to backup files");
            },
            Some(Commands::Restore { vault, target, snapshot }) => {
                let password = ask_for_password();

                let backup_vault = match BackupVault::open(vault, &password) {
                    Ok(vault) => vault,
                    Err(BackupError::VaultWrongPassword) => {
                        println!("Wrong password");
                        std::process::exit(1);
                    },
                    Err(_) => {
                        println!("Failed to open vault");
                        std::process::exit(1);
                    }
                };

                backup_vault.restore(vault, snapshot, target);
            },
            None => {
                Cli::command().print_help().unwrap();
            }
        }
    }
}

fn ask_for_password() -> String {
    print!("Enter the password for the backup vault: ");
    std::io::stdout().flush().unwrap();
    let mut password = String::new();
    std::io::stdin().read_line(&mut password).unwrap();
    password.trim().to_string()
}

fn _naive_copy_file(input: &PathBuf, output_dir: &PathBuf) -> std::io::Result<()> {
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

fn _naive_copy_dir(input_dir: &PathBuf, output_dir: &PathBuf) -> std::io::Result<()> {
    // Create the output directory if it doesn't exist
    fs::create_dir_all(&output_dir)?;

    for entry in fs::read_dir(&input_dir)? {
        let entry = entry?;

        if entry.file_type()?.is_dir() {
            let dir_name = entry.file_name();
            let dir_path = entry.path();

            let dest_path = output_dir.join(dir_name);

            _naive_copy_dir(&dir_path, &dest_path)?;
        } else if entry.file_type()?.is_file() {
            let file_name = entry.file_name();
            let file_path = entry.path();

            let dest_path = output_dir.join(file_name);

            fs::copy(&file_path, &dest_path)?;
        }
    }

    Ok(())
}