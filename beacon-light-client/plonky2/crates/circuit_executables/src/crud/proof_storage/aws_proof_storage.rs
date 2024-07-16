use anyhow::Result;
use async_trait::async_trait;
use aws_config::{BehaviorVersion, Region};
use aws_sdk_s3::{primitives::ByteStream, Client, Config};

use super::proof_storage::ProofStorage;

pub struct AwsStorage {
    client: Client,
    bucket_name: String,
}

impl AwsStorage {
    pub async fn new(region: String, bucket_name: String) -> AwsStorage {
        let aws_config = aws_config::defaults(BehaviorVersion::latest()).load().await;

        let s3_config = Config::builder()
            .credentials_provider(aws_config.credentials_provider().unwrap())
            .behavior_version_latest()
            .region(Region::new(region))
            .force_path_style(true)
            .clone()
            .build();

        let client = Client::from_conf(s3_config);

        AwsStorage {
            client,
            bucket_name,
        }
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
