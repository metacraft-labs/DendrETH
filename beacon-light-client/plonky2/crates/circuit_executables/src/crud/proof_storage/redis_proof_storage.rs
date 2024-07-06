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

#[async_trait]
impl ProofStorage for RedisStorage {
    async fn get_proof(&mut self, key: String) -> Result<Vec<u8>> {
        Ok(self.connection.get(&key).await?)
    }

    async fn set_proof(&mut self, key: String, proof: &[u8]) -> Result<()> {
        Ok(self.connection.set(&key, proof).await?)
    }

    async fn del_proof(&mut self, key: String) -> Result<()> {
        Ok(self.connection.del(&key).await?)
    }

    async fn get_keys_count(&mut self, pattern: String) -> usize {
        let result: Vec<String> = self.connection.keys(pattern).await.unwrap();
        result.len()
    }
}
