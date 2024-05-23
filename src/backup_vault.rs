extern crate time;

use std::fs;
use std::fs::File;
use std::path::PathBuf;
use std::io::{Read, Write};
use std::time::{SystemTime, UNIX_EPOCH};

use uuid::Uuid;
use serde::{Serialize, Deserialize};

const BUF_SIZE: usize = 4*1024*1024;

#[derive(Debug)]
pub enum BackupError {
    VaultDoesNotExist,
    VaultReadError,
    VaultCreationError,
    VaultWrongPassword,
    VaultFileOpenError,
    VaultFileReadError,
    VaultFileCopyError,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultFile {
    pub file_name: String,
    pub file_hash: String,
    pub file_path: PathBuf,
    pub file_size: u64,
    pub vault_paths: Vec<PathBuf>,
}

#[derive(Serialize, Deserialize)]
pub struct Snapshot {
    pub snapshot_id: String,
    pub snapshot_time: String,
    pub snapshot_files: Vec<VaultFile>
}

#[derive(Serialize, Deserialize)]
pub struct VaultConfig {
    pub password_hash: String,
}

#[derive(Serialize, Deserialize)]
pub struct BackupVault {
    pub vault_path: PathBuf,
    pub password: String,
    pub snapshots: Vec<Snapshot>,
    pub files: Vec<VaultFile>,
}

impl BackupVault {
    pub fn new(vault_path: PathBuf, password: String) -> Self {
        Self {
            vault_path,
            password,
            snapshots: vec![],
            files: vec![],
        }
    }

    pub fn create(vault_path: &PathBuf, password: &String) -> Result<BackupVault, BackupError> {
        if vault_path.exists() && vault_path.is_dir() && !is_directory_empty(&vault_path) {
            println!("Vault already exists and is not empty");
            return Err(BackupError::VaultCreationError);
        }

        if !vault_path.exists() {
            let create_result = fs::create_dir_all(vault_path);

            if create_result.is_err() {
                println!("Failed to create vault directory, {}", create_result.err().unwrap());
                return Err(BackupError::VaultCreationError);
            }
        }

        let config_path = vault_path.join("vault_config.json");

        
        let hash = bcrypt::hash(password, bcrypt::DEFAULT_COST).unwrap();        
        let config = VaultConfig {
            password_hash: hash,
        };

        let config_file = File::create(config_path);

        if config_file.is_err() {
            println!("Failed to create config file, {}", config_file.err().unwrap());
            return Err(BackupError::VaultCreationError);
        }

        let mut config_file = config_file.unwrap();

        let config_json_data = serde_json::to_string(&config);

        if config_json_data.is_err() {
            println!("Failed to serialize config data");
            return Err(BackupError::VaultCreationError);
        }

        let config_json_data = config_json_data.unwrap();

        let write_result = config_file.write_all(config_json_data.as_bytes());

        if write_result.is_err() {
            println!("Failed to write config data");
            return Err(BackupError::VaultCreationError);
        }

        Ok(BackupVault::new(vault_path.clone(), password.clone()))

    }

    pub fn open(vault_path: &PathBuf, password: &String) -> Result<BackupVault, BackupError> {
        if !vault_path.exists() || !vault_path.is_dir() {
            return Err(BackupError::VaultDoesNotExist);
        }

        let config_path = vault_path.join("vault_config.json");

        if !config_path.exists() {
            return Err(BackupError::VaultReadError);
        }

        let config_file = File::open(config_path);

        if config_file.is_err() {
            return Err(BackupError::VaultReadError);
        }

        let config_file = config_file.unwrap();

        let config: VaultConfig = match serde_json::from_reader(config_file) {
            Ok(config) => config,
            Err(_) => return Err(BackupError::VaultReadError),
        };

        let is_password_correct = bcrypt::verify(password, &config.password_hash);

        if is_password_correct.is_err() {
            return Err(BackupError::VaultReadError);
        }
        let is_password_correct = is_password_correct.unwrap();

        if !is_password_correct {
            println!("Wrong password:\n{}\n{} != {}", password, config.password_hash, bcrypt::hash(password, bcrypt::DEFAULT_COST).unwrap());
            return Err(BackupError::VaultWrongPassword);
        }

        let vault_file = File::open(vault_path.join("vault.json"));

        if vault_file.is_err() {
            return Err(BackupError::VaultReadError);
        }

        let vault_file = vault_file.unwrap();

        let vault: BackupVault = match serde_json::from_reader(vault_file) {
            Ok(vault) => vault,
            Err(_) => return Err(BackupError::VaultReadError),
        };

        Ok(BackupVault {
            vault_path: vault_path.clone(),
            password: password.clone(),
            snapshots: vault.snapshots,
            files: vault.files,
        })
    }

    fn vault_copy_file(&mut self, file: &mut VaultFile) -> Result<(), BackupError> {
        println!("File name {}\nFile hash {}\nFile size {}\n\n", file.file_name, file.file_hash, file.file_size);

        // // let hash = &file.file_hash;
        // // let vault_file_name = format!("{}{}", hash[0..8].to_string(), hash[hash.len() - 8..hash.len()].to_string());
        
        let read_file = File::open(&file.file_path);

        if read_file.is_err() {
            println!("Failed to open file: {}", file.file_name);
            return Err(BackupError::VaultFileOpenError);
        }

        let mut read_file = read_file.unwrap();
        // let mut file_buf = [0; BUF_SIZE];
        
        loop {
            // let read_result = read_file.read(&mut file_buf);
            let mut buffer = Vec::new();
            let read_result = read_file.read_to_end(&mut buffer);

            if read_result.is_err() {
                println!("Failed to read file: {}", file.file_name);
                return Err(BackupError::VaultFileReadError);
            }

            let read_result = read_result.unwrap();

            if read_result == 0 {
                break;
            }

            let mut hasher = blake3::Hasher::new();
            let hash = hasher.update(&buffer[..read_result]).finalize().to_hex().to_string();

            let write_file_path = self.vault_path.join(hash[0..16].to_string());

            let vault_file = File::create(&write_file_path);

            if vault_file.is_err() {
                println!("Failed to create file: {}", write_file_path.to_str().unwrap());
                return Err(BackupError::VaultFileCopyError);
            }

            let mut vault_file = vault_file.unwrap();

            let write_result = vault_file.write_all(&buffer[..read_result]);

            if write_result.is_err() {
                println!("Failed to write file: {}", write_file_path.to_str().unwrap());
                return Err(BackupError::VaultFileCopyError);
            }

            println!("Copied file: {}", write_file_path.to_str().unwrap());

            file.vault_paths.push(write_file_path);
        }

        return Ok(());
    }

    fn vault_add_file(&mut self, file_path: &PathBuf) -> Option<VaultFile> {
        
        let file = File::open(&file_path);

        if file.is_err() {
            println!("Failed to open file: {}", file_path.to_str().unwrap());
            return None;
        } 
        
        let file_name = file_path.file_name().unwrap().to_str().unwrap().to_string();
        let file_size = file_path.metadata().unwrap().len();
        
        let mut file = file.unwrap();

        let mut file_buf = [0; BUF_SIZE];

        let mut file_hasher = blake3::Hasher::new();

        loop {
            let read_result = file.read(&mut file_buf);

            if read_result.is_err() {
                println!("Failed to read file: {}", file_path.to_str().unwrap());
                break;
            }

            let read_result = read_result.unwrap();

            if read_result == 0 {
                break;
            }

            file_hasher.update(&file_buf[..read_result]);
        }

        
        let file_hash = file_hasher.finalize().to_hex().to_string();

        let mut vault_file = VaultFile {
            file_name,
            file_hash,
            file_path: file_path.clone(),
            file_size,
            vault_paths: vec![],
        };


        if self.files.is_empty() || !self.files.iter().any(|f| f.file_hash == vault_file.file_hash) {
            self.vault_copy_file(&mut vault_file).expect("Failed to copy file")
        }
        
        return Some(vault_file);
    }

    pub fn backup(&mut self, files_path: &Vec<PathBuf>) -> Result<(), BackupError>{
        println!("Creating backup...");

        let snapshot_id = Uuid::new_v4();
        let sys_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

        let mut snapshot = Snapshot {
            snapshot_id: snapshot_id.to_string(),
            snapshot_time: sys_time.to_string(),
            snapshot_files: vec![],
        };

        let files_path = files_path.iter().flat_map(expand_file_path).collect::<Vec<PathBuf>>();

        let vault_files: Vec<Option<VaultFile>> = files_path.iter().map(|file_path| self.vault_add_file(file_path)).collect();
        let vault_files: Vec<VaultFile> = vault_files.into_iter().filter_map(|file| file).collect();

        for vault_file in vault_files {
            snapshot.snapshot_files.push(vault_file);
        }

        self.snapshots.push(snapshot);
        println!("Snapshot created {}", snapshot_id);

        let vault_file = File::create(self.vault_path.join("vault.json"));

        if vault_file.is_err() {
            println!("Failed to create vault file");
            return Err(BackupError::VaultFileOpenError);
        }

        let mut vault_file = vault_file.unwrap();

        let temp_password = self.password.clone();
        self.password = "".to_string();
        let vault_json_data = serde_json::to_string(&self);
        self.password = temp_password;

        if vault_json_data.is_err() {
            println!("Failed to serialize vault data");
            return Err(BackupError::VaultFileOpenError);
        }

        let vault_json_data = vault_json_data.unwrap();

        let write_result = vault_file.write_all(vault_json_data.as_bytes());

        if write_result.is_err() {
            println!("Failed to write vault data");
            return Err(BackupError::VaultFileOpenError);
        }

        Ok(())
    }

    pub fn restore(&self, vault: &PathBuf, snapshot: &Option<String>, target: &PathBuf) {
        println!("Restoring backup...");
    
        let vault = BackupVault::open(vault, &self.password).expect("Failed to open vault");

        let snapshot = match snapshot {
            Some(snapshot) => vault.snapshots.iter().find(|s| s.snapshot_id == snapshot.clone()).expect("Snapshot not found"),
            None => vault.snapshots.last().expect("No snapshots found"),
        };

        fs::create_dir_all(target).expect("Failed to create target directory");

        for vault_file in &snapshot.snapshot_files {
            let file_path = target.join(&vault_file.file_name);
            
            if file_path.parent().is_some() {
                fs::create_dir_all(&file_path.parent().unwrap()).expect("Failed to create directory");
            }

            let mut file = File::create(&file_path).expect("Failed to create file");

            for vault_path in &vault_file.vault_paths {
                let mut vault_file = File::open(vault_path).expect("Failed to open vault file");
                let mut buffer = Vec::new();
                vault_file.read_to_end(&mut buffer).expect("Failed to read vault file");
                file.write_all(&buffer).expect("Failed to write file");
            }
        }

    }

}

fn is_directory_empty(path: &PathBuf) -> bool {
    match fs::read_dir(path) {
        Ok(mut dir) => dir.next().is_none(),
        Err(_) => true
    }
}

fn expand_file_path(file_path: &PathBuf) -> Vec<PathBuf> {
    let mut result = vec![];

    if file_path.is_dir() {
        for entry in fs::read_dir(file_path).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_dir() {
                result.extend(expand_file_path(&path));
            } else if path.is_file() {
                result.push(path);
            }
        }
    } else if file_path.is_file() {
        result.push(file_path.clone());
    }

    result
}