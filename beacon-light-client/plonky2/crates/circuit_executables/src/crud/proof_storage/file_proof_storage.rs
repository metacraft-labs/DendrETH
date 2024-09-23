use super::proof_storage::ProofStorage;
use anyhow::Result;
use async_trait::async_trait;
use glob::glob;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct FilesystemBlobStorageDefinition {
    pub directory: String,
}

pub struct FileStorage {
    folder_name: String,
}

unsafe impl Send for FileStorage {}
unsafe impl Sync for FileStorage {}

impl FileStorage {
    pub fn new(folder_name: String) -> FileStorage {
        if !fs::metadata(&folder_name).is_ok() {
            fs::create_dir_all(&folder_name).unwrap();
        }

        FileStorage { folder_name }
    }
}

#[async_trait]
impl ProofStorage for FileStorage {
    async fn get_proof(&mut self, key: String) -> Result<Vec<u8>> {
        Ok(fs::read(format!("{}/{}", self.folder_name, key))?)
    }

    async fn set_proof(&mut self, key: String, proof: &[u8]) -> Result<()> {
        Ok(fs::write(format!("{}/{}", self.folder_name, key), proof)?)
    }

    async fn del_proof(&mut self, key: String) -> Result<()> {
        Ok(fs::remove_file(format!("{}/{}", self.folder_name, key))?)
    }

    async fn get_keys_count(&mut self, pattern: String) -> usize {
        glob(&format!("{}/{}", self.folder_name, pattern))
            .unwrap()
            .filter(|path| matches!(path, Ok(_)))
            .count()
    }
}
