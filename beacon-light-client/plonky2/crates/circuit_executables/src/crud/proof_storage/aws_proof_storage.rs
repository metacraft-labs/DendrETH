use anyhow::Result;
use async_trait::async_trait;
use aws_config::{default_provider::credentials, BehaviorVersion, ConfigLoader, Region};
use aws_sdk_s3::{config::Credentials, primitives::ByteStream, Client};
use serde::{Deserialize, Serialize};
use std::fs;

use crate::crud::common::read_file_to_string;

use super::proof_storage::ProofStorage;

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct S3Credentials {
    pub access_key_id: String,
    pub secret_access_key_filepath: String,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct S3BlobStorageDefinition {
    pub region: String,
    pub bucket_name: String,
    pub endpoint_url: Option<String>,
    pub credentials: S3Credentials,
}

pub struct AwsStorage {
    client: Client,
    bucket_name: String,
}

impl AwsStorage {
    pub async fn new(
        region: String,
        bucket_name: String,
        endpoint_url: Option<String>,
        credentials: &S3Credentials,
    ) -> Result<AwsStorage> {
        let secret_access_key = read_file_to_string(&credentials.secret_access_key_filepath)?;

        let credentials = Credentials::new(
            &credentials.access_key_id,
            &secret_access_key,
            None,
            None,
            "custom_provider",
        );

        let credentials_provider = credentials::DefaultCredentialsChain::builder()
            .with_custom_credential_source("custom_credentials_source", credentials)
            .build()
            .await;

        let mut config_loader = ConfigLoader::default()
            .behavior_version(BehaviorVersion::latest())
            .credentials_provider(credentials_provider)
            .region(Region::new(region));

        if let Some(url) = endpoint_url {
            config_loader = config_loader.endpoint_url(url);
        }

        Ok(AwsStorage {
            client: Client::new(&config_loader.load().await),
            bucket_name,
        })
    }
}

#[async_trait]
impl ProofStorage for AwsStorage {
    async fn get_proof(&mut self, key: String) -> Result<Vec<u8>> {
        let resp = self
            .client
            .get_object()
            .bucket(self.bucket_name.clone())
            .key(key)
            .send()
            .await?;

        let body = resp.body.collect().await?;
        let content = body.into_bytes().to_vec();

        Ok(content)
    }

    async fn set_proof(&mut self, key: String, proof: &[u8]) -> Result<()> {
        let byte_stream = ByteStream::from(proof.to_vec());

        self.client
            .put_object()
            .bucket(self.bucket_name.clone())
            .key(key)
            .body(byte_stream)
            .send()
            .await?;

        Ok(())
    }

    async fn del_proof(&mut self, key: String) -> Result<()> {
        self.client
            .delete_object()
            .bucket(self.bucket_name.clone())
            .key(key)
            .send()
            .await?;

        Ok(())
    }

    async fn get_keys_count(&mut self, pattern: String) -> usize {
        let pattern = glob::Pattern::new(&pattern).unwrap();

        let mut response = self
            .client
            .list_objects_v2()
            .bucket(self.bucket_name.clone())
            .max_keys(10) // In this example, go 10 at a time.
            .into_paginator()
            .send();

        let mut count = 0;

        while let Some(Ok(result)) = response.next().await {
            count += result
                .contents()
                .iter()
                .filter(|&item| pattern.matches(item.key().unwrap()))
                .count();
        }

        count
    }
}
