use super::proof_storage::ProofStorage;
use anyhow::Result;
use async_trait::async_trait;
use glob::glob;
use std::fs;

pub struct FileStorage {
    folder_name: String,
}

impl FileStorage {
    pub fn new(folder_name: String) -> FileStorage {
        if !fs::metadata(&folder_name).is_ok() {
            fs::create_dir_all(&folder_name).unwrap();
        }

        FileStorage { folder_name }
    }
}

#[async_trait(?Send)]
impl ProofStorage for FileStorage {
    async fn get_proof(&mut self, identifier: String) -> Result<Vec<u8>> {
        let result = fs::read(format!("{}/{}", self.folder_name, identifier)).unwrap();

        Ok(result)
    }

    async fn set_proof(&mut self, identifier: String, proof: &[u8]) -> Result<()> {
        fs::write(format!("{}/{}", self.folder_name, identifier), proof).unwrap();

        Ok(())
    }

    async fn del_proof(&mut self, identifier: String) -> Result<()> {
        Ok(fs::remove_file(format!(
            "{}/{}",
            self.folder_name, identifier
        ))?)
    }

    async fn get_keys_count(&mut self, pattern: String) -> usize {
        glob(&format!("{}/{}", self.folder_name, pattern))
            .unwrap()
            .filter(|path| matches!(path, Ok(_)))
            .count()
    }
}
