use std::{env, str::FromStr};

use async_trait::async_trait;
use clap::ArgMatches;

use anyhow::Result;

use super::{
    aws_proof_storage::AwsStorage, azure_proof_storage::AzureStorage,
    file_proof_storage::FileStorage, redis_proof_storage::RedisStorage,
};

#[async_trait(?Send)]
pub trait ProofStorage {
    async fn get_proof(&mut self, identifier: String) -> Result<Vec<u8>>;

    async fn set_proof(&mut self, identifier: String, proof: &[u8]) -> Result<()>;

    async fn del_proof(&mut self, identifier: String) -> Result<()>;

    async fn get_keys_count(&mut self, pattern: String) -> usize;
}

pub enum ProofStorageType {
    Redis,
    File,
    Azure,
    Aws,
}

impl FromStr for ProofStorageType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "redis" => Ok(ProofStorageType::Redis),
            "file" => Ok(ProofStorageType::File),
            "azure" => Ok(ProofStorageType::Azure),
            "aws" => Ok(ProofStorageType::Aws),
            _ => Err(()),
        }
    }
}

pub async fn create_proof_storage(args: &ArgMatches) -> Box<dyn ProofStorage> {
    let proof_storage_type = args
        .value_of("proof_storage_type")
        .unwrap()
        .parse::<ProofStorageType>()
        .unwrap();

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
            dotenv::dotenv().ok();

            let access_key = env::var("STORAGE_ACCESS_KEY").expect("missing STORAGE_ACCOUNT_KEY");

            Box::new(AzureStorage::new(
                args.value_of("azure_account").unwrap().to_string(),
                access_key,
                args.value_of("azure_container").unwrap().to_string(),
            ))
        }
        ProofStorageType::Aws => Box::new(
            AwsStorage::new(
                args.value_of("aws_endpoint_url").unwrap().to_string(),
                args.value_of("aws_region").unwrap().to_string(),
                args.value_of("aws_bucket_name").unwrap().to_string(),
            )
            .await,
        ),
    }
}
