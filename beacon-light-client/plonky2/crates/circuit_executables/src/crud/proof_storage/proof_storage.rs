use std::{collections::HashMap, fs};

use async_trait::async_trait;

use anyhow::{Context, Result};
use redis::aio::Connection;
use serde::{Deserialize, Serialize};

use super::{
    aws_proof_storage::{AwsStorage, S3BlobStorageDefinition},
    file_proof_storage::{FileStorage, FilesystemBlobStorageDefinition},
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
    pub auth: Option<String>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(tag = "type")]
pub enum BlobStorageDefinition {
    S3(S3BlobStorageDefinition),
    Filesystem(FilesystemBlobStorageDefinition),
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct RedisBlobStorageDefinition {
    pub blob_storage: BlobStorageDefinition,
    pub metadata_storage: RedisConnectionDefinition,
}

pub struct RedisBlobStorage {
    pub blob: Box<dyn ProofStorage>,
    pub metadata: Connection,
}

impl RedisBlobStorage {
    pub async fn from_definition(def: &RedisBlobStorageDefinition) -> Result<Self> {
        Ok(Self {
            blob: blob_storage_from_definition(&def.blob_storage).await?,
            metadata: redis_connection_from_definition(&def.metadata_storage).await?,
        })
    }

    pub async fn from_config(cfg: &ProofStorageConfig, storage_name: &str) -> Result<Self> {
        let def = proof_storage_definition_from_config(cfg, storage_name)?;
        Ok(Self::from_definition(def).await?)
    }

    pub async fn from_file(filepath: &str, storage_name: &str) -> Result<Self> {
        let config = load_storage_config(filepath)?;
        Ok(Self::from_config(&config, storage_name).await?)
    }
}

pub async fn blob_storage_from_definition(
    def: &BlobStorageDefinition,
) -> Result<Box<dyn ProofStorage>> {
    Ok(match def {
        BlobStorageDefinition::S3(cfg) => Box::new(
            AwsStorage::new(
                cfg.region.clone(),
                cfg.bucket_name.clone(),
                cfg.endpoint_url.clone(),
                &cfg.credentials,
            )
            .await,
        ),
        BlobStorageDefinition::Filesystem(cfg) => Box::new(FileStorage::new(cfg.directory.clone())),
    })
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
) -> Result<&'a RedisBlobStorageDefinition> {
    Ok(cfg
        .get(storage_name)
        .context("Proof storage is not in config")?)
}

pub fn load_storage_config(filepath: &str) -> Result<ProofStorageConfig> {
    Ok(serde_json::from_str(&fs::read_to_string(filepath)?)?)
}

pub async fn redis_connection_from_definition(
    def: &RedisConnectionDefinition,
) -> Result<Connection> {
    let url = redis_url_from_definition(def);
    let client = redis::Client::open(url)?;
    Ok(client.get_async_connection().await?)
}

pub fn redis_url_from_definition(def: &RedisConnectionDefinition) -> String {
    let at = if def.auth.as_ref().is_some_and(|x| x.len() > 0) {
        format!("{}@", def.auth.as_ref().unwrap())
    } else {
        String::new()
    };
    format!("redis://{}{}:{}", at, def.host, def.port)
}

pub type ProofStorageConfig = HashMap<String, RedisBlobStorageDefinition>;
