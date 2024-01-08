use std::env;

use async_trait::async_trait;
use clap::ArgMatches;

use anyhow::Result;

use super::{file_proof_storage::FileStorage, redis_proof_storage::RedisStorage};

#[async_trait(?Send)]
pub trait ProofStorage {
    async fn get_proof(&mut self, identifier: String) -> Result<Vec<u8>>;

    async fn set_proof(&mut self, identifier: String, proof: &[u8]) -> Result<()>;
}

pub enum ProofStorageType {
    Redis,
    File,
    Azure,
    Aws,
}

pub async fn create_proof_storage(
    proof_storage_type: ProofStorageType,
    args: ArgMatches,
) -> Box<dyn ProofStorage> {
    match proof_storage_type {
        ProofStorageType::Redis => {
            let redis_connection = args.value_of("redis_connection").unwrap();

            Box::new(
                RedisStorage::new(redis_connection.to_string())
                    .await
                    .unwrap(),
            )
        }
        ProofStorageType::File => {
            let folder_name = args.value_of("folder_name").unwrap();

            Box::new(FileStorage::new(folder_name.to_string()))
        }
        ProofStorageType::Azure => {
            let access_key = env::var("STORAGE_ACCESS_KEY").expect("missing STORAGE_ACCOUNT_KEY");

            todo!()
        }
        ProofStorageType::Aws => {
            todo!()
        }
    }
}
