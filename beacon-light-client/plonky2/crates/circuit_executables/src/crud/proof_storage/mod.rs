pub mod aws_proof_storage;
pub mod azure_proof_storage;
pub mod file_proof_storage;
pub mod redis_proof_storage;

use crate::crud::proof_storage::{aws_proof_storage::AwsStorage, file_proof_storage::FileStorage};
use std::{collections::HashMap, fs};

use async_trait::async_trait;

use anyhow::{ensure, Context, Result};
use redis::aio::Connection;
use serde::{Deserialize, Serialize};

use crate::crud::{
    common::read_file_to_string,
    proof_storage::{
        aws_proof_storage::S3BlobStorageDefinition,
        file_proof_storage::FilesystemBlobStorageDefinition, redis_proof_storage::RedisStorage,
    },
};

#[async_trait]
pub trait ProofStorage: Send + Sync {
    async fn get_proof(&mut self, key: String) -> Result<Vec<u8>>;

    async fn set_proof(&mut self, key: String, proof: &[u8]) -> Result<()>;

    async fn del_proof(&mut self, key: String) -> Result<()>;

    async fn get_keys_count(&mut self, pattern: String) -> usize;
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct RedisConnectionDefinition {
    pub host: String,
    pub port: u64,
    pub auth_filepath: Option<String>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(tag = "type")]
pub enum BlobStorageDefinition {
    S3(S3BlobStorageDefinition),
    Filesystem(FilesystemBlobStorageDefinition),
    Redis(RedisConnectionDefinition),
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct MetadataBlobStorageDefinition {
    pub blob_storage: BlobStorageDefinition,
    pub metadata_storage: RedisConnectionDefinition,
}

pub struct MetadataBlobStorage {
    pub blob: Box<dyn ProofStorage>,
    pub metadata: Connection,
}

impl MetadataBlobStorage {
    pub async fn from_definition(def: &MetadataBlobStorageDefinition) -> Result<Self> {
        Ok(Self {
            blob: blob_storage_from_definition(&def.blob_storage).await?,
            metadata: redis_connection_from_definition(&def.metadata_storage).await?,
        })
    }

    pub async fn from_config(cfg: &ProofStorageConfig, storage_name: &str) -> Result<Self> {
        let def = proof_storage_definition_from_config(cfg, storage_name)?;
        Self::from_definition(def).await
    }

    pub async fn from_file(filepath: &str, storage_name: &str) -> Result<Self> {
        let config = load_storage_config(filepath)?;
        Self::from_config(&config, storage_name).await
    }
}

pub async fn blob_storage_from_definition(
    def: &BlobStorageDefinition,
) -> Result<Box<dyn ProofStorage>> {
    let storage: Box<dyn ProofStorage> = match def {
        BlobStorageDefinition::S3(cfg) => Box::new(
            AwsStorage::new(
                cfg.region.clone(),
                cfg.bucket_name.clone(),
                cfg.endpoint_url.clone(),
                &cfg.credentials,
            )
            .await?,
        ),
        BlobStorageDefinition::Filesystem(cfg) => Box::new(FileStorage::new(cfg.directory.clone())),
        BlobStorageDefinition::Redis(cfg) => {
            let url = redis_url_from_definition(cfg)?;
            let storage = RedisStorage::new(url).await?;
            Box::new(storage)
        }
    };

    Ok(storage)
}

pub async fn proof_storage_from_config<'a>(
    cfg: &'a ProofStorageConfig,
    storage_name: &str,
) -> Result<Box<dyn ProofStorage>> {
    let def = &proof_storage_definition_from_config(cfg, storage_name)?.blob_storage;
    blob_storage_from_definition(def).await
}

pub fn proof_storage_definition_from_config<'a>(
    cfg: &'a ProofStorageConfig,
    storage_name: &str,
) -> Result<&'a MetadataBlobStorageDefinition> {
    cfg.get(storage_name)
        .with_context(|| format!("Proof storage `{storage_name}` is not in config"))
}

pub fn load_storage_config(filepath: &str) -> Result<ProofStorageConfig> {
    Ok(serde_json::from_str(&fs::read_to_string(filepath)?)?)
}

pub async fn redis_connection_from_definition(
    def: &RedisConnectionDefinition,
) -> Result<Connection> {
    let url = redis_url_from_definition(def)?;
    let client = redis::Client::open(url)?;
    Ok(client.get_async_connection().await?)
}

pub fn redis_url_from_definition(def: &RedisConnectionDefinition) -> Result<String> {
    let auth = match &def.auth_filepath {
        Some(path) => format!("{}@", read_file_to_string(path)?),
        None => String::new(),
    };
    ensure!(auth != "@", "Redis authentication string is empty");

    Ok(format!("redis://{}{}:{}", auth, def.host, def.port))
}

pub type ProofStorageConfig = HashMap<String, MetadataBlobStorageDefinition>;
