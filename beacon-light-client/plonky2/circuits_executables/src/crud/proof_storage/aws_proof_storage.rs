use anyhow::Result;
use async_trait::async_trait;
use aws_config::Region;
use aws_sdk_s3::{primitives::ByteStream, Client, Config};

use super::proof_storage::ProofStorage;

pub struct AwsStorage {
    client: Client,
    bucket_name: String,
}

impl AwsStorage {
    pub async fn new(endpoint_url: String, region: String, bucket_name: String) -> AwsStorage {
        let aws_config = aws_config::from_env().load().await;

        let s3_config = Config::builder()
            .credentials_provider(aws_config.credentials_provider().unwrap())
            .behavior_version_latest()
            .region(Region::new(region))
            .endpoint_url(endpoint_url)
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

#[async_trait(?Send)]
impl ProofStorage for AwsStorage {
    async fn get_proof(&mut self, identifier: String) -> Result<Vec<u8>> {
        let resp = self
            .client
            .get_object()
            .bucket(self.bucket_name)
            .key(identifier)
            .send()
            .await?;

        let mut body = resp.body.collect().await?;
        let mut content = body.into_bytes().to_vec();

        Ok(content)
    }

    async fn set_proof(&mut self, identifier: String, proof: &[u8]) -> Result<()> {
        let byte_stream = ByteStream::from(proof.to_vec());

        self.client
            .put_object()
            .bucket(self.bucket_name)
            .key(identifier)
            .body(byte_stream)
            .send()
            .await
            .unwrap();

        Ok(())
    }
}
