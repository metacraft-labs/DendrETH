use super::proof_storage::ProofStorage;
use anyhow::Result;
use async_trait::async_trait;
use azure_storage::StorageCredentials;
use azure_storage_blobs::prelude::{ClientBuilder, ContainerClient};
use futures::StreamExt;

pub struct AzureStorage {
    container_client: ContainerClient,
}

impl AzureStorage {
    pub fn new(account: String, access_key: String, container: String) -> AzureStorage {
        let storage_credentials =
            StorageCredentials::access_key(account.clone(), access_key.clone());

        let container_client =
            ClientBuilder::new(account, storage_credentials).container_client(&container);

        AzureStorage { container_client }
    }
}

#[async_trait(?Send)]
impl ProofStorage for AzureStorage {
    async fn get_proof(&mut self, identifier: String) -> Result<Vec<u8>> {
        let blob_client = self.container_client.blob_client(identifier);

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

    async fn set_proof(&mut self, identifier: String, proof: &[u8]) -> Result<()> {
        let blob_client = self.container_client.blob_client(identifier);

        blob_client
            .put_block_blob(proof.to_vec())
            .content_type("application/octet-stream")
            .await?;

        Ok(())
    }
}
