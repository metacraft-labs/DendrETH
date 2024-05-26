use std::env;

use super::proof_storage::ProofStorage;
use anyhow::Result;
use async_trait::async_trait;
use azure_storage::ConnectionString;

use azure_storage_blobs::{container::operations::BlobItem, prelude::*};
use futures::StreamExt;

pub struct AzureStorage {
    container_client: ContainerClient,
}

impl AzureStorage {
    pub fn new(container: String) -> AzureStorage {
        let connection_string = env::var("AZURE_CONNECTION_STRING").unwrap();
        let account = env::var("STORAGE_ACCOUNT").unwrap();

        let container_client = ClientBuilder::new(
            account,
            ConnectionString::new(connection_string.as_str())
                .unwrap()
                .storage_credentials()
                .unwrap(),
        )
        .container_client(&container);

        AzureStorage { container_client }
    }
}

#[async_trait(?Send)]
impl ProofStorage for AzureStorage {
    async fn get_proof(&mut self, key: String) -> Result<Vec<u8>> {
        let blob_client = self.container_client.blob_client(key);

        let result = blob_client
            .get()
            .into_stream()
            .next()
            .await
            .unwrap()
            .unwrap();

        let bytes_result = result.data.collect().await.unwrap().to_vec();

        Ok(bytes_result)
    }

    async fn set_proof(&mut self, key: String, proof: &[u8]) -> Result<()> {
        let blob_client = self.container_client.blob_client(key);

        blob_client
            .put_block_blob(proof.to_vec())
            .content_type("application/octet-stream")
            .await?;

        Ok(())
    }

    async fn del_proof(&mut self, key: String) -> Result<()> {
        let blob_client = self.container_client.blob_client(key);
        blob_client.delete().await?;
        Ok(())
    }

    async fn get_keys_count(&mut self, pattern: String) -> usize {
        let pattern = glob::Pattern::new(&pattern).unwrap();
        let mut iter = self.container_client.list_blobs().into_stream();

        let mut count = 0;

        while let Some(Ok(reponse)) = iter.next().await {
            count += reponse
                .blobs
                .items
                .iter()
                .filter(|&item| {
                    if let BlobItem::Blob(blob) = item {
                        pattern.matches(&blob.name)
                    } else {
                        false
                    }
                })
                .count();
        }

        count
    }
}
