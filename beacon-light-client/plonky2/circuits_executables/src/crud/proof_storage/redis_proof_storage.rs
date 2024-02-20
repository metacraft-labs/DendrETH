use super::proof_storage::ProofStorage;
use anyhow::Result;
use async_trait::async_trait;
use redis::{aio::Connection, AsyncCommands};

pub struct RedisStorage {
    connection: Connection,
}

impl RedisStorage {
    pub async fn new(connection_string: String) -> Result<RedisStorage, redis::RedisError> {
        let client = redis::Client::open(connection_string)?;
        let connection = client.get_async_connection().await?;
        Ok(RedisStorage { connection })
    }
}

#[async_trait(?Send)]
impl ProofStorage for RedisStorage {
    async fn get_proof(&mut self, identifier: String) -> Result<Vec<u8>> {
        Ok(self.connection.get(&identifier).await?)
    }

    async fn set_proof(&mut self, identifier: String, proof: &[u8]) -> Result<()> {
        Ok(self.connection.set(&identifier, proof).await?)
    }

    async fn del_proof(&mut self, identifier: String) -> Result<()> {
        Ok(self.connection.del(&identifier).await?)
    }

    async fn get_keys_count(&mut self, pattern: String) -> usize {
        let result: Vec<String> = self.connection.keys(pattern).await.unwrap();
        result.len()
    }
}
