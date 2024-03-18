use std::path::PathBuf;

pub struct VaultFile {
    pub file_name: String,
    pub file_hash: String,
    pub file_path: PathBuf,
    pub file_size: u64,
    pub vault_paths: Vec<PathBuf>,
}

pub struct Snapshot {
    pub snapshot_id: String,
    pub snapshot_time: String,
    pub snapshot_files: Vec<VaultFile>
}

pub struct VaultConfig {
    pub vault_path: PathBuf,
}
pub struct BackupVault {
    pub vault_path: PathBuf,
    pub password: &'static str,
    pub snapshots: Vec<Snapshot>,
}

impl BackupVault {
    pub fn new(vault_path: PathBuf, password: &'static str) -> Self {
        Self {
            vault_path,
            password,
            snapshots: vec![],
        }
    }
}

