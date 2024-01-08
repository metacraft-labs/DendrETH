use super::proof_storage::ProofStorage;
use anyhow::Result;
use async_trait::async_trait;
use redis::aio::Connection;
use redis::AsyncCommands;

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
        let result: Vec<u8> = self.connection.get(&identifier).await?;

        Ok(result)
    }

    async fn set_proof(&mut self, identifier: String, proof: &[u8]) -> Result<()> {
        self.connection.set(&identifier, proof).await?;

        Ok(())
    }
}
