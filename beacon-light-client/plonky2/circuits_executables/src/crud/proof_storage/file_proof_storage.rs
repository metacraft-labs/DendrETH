use super::proof_storage::ProofStorage;
use anyhow::Result;
use async_trait::async_trait;
use std::fs;

pub struct FileStorage;

impl FileStorage {
    pub fn new() -> FileStorage {
        if !fs::metadata("proofs").is_ok() {
            fs::create_dir_all("proofs").unwrap();
        }

        FileStorage
    }
}

#[async_trait(?Send)]
impl ProofStorage for FileStorage {
    async fn get_proof(&mut self, identifier: String) -> Result<Vec<u8>> {
        let result = fs::read(format!("{}/{}.{}", "proofs", identifier, "bin")).unwrap();

        Ok(result)
    }

    async fn set_proof(&mut self, identifier: String, proof: &[u8]) -> Result<()> {
        fs::write(format!("{}/{}.{}", "proofs", identifier, "bin"), proof).unwrap();

        Ok(())
    }
}
