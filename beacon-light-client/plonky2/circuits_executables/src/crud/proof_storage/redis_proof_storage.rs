use super::proof_storage::ProofStorage;
use anyhow::Result;
use async_trait::async_trait;
use redis::aio::Connection;
use redis::AsyncCommands;

pub struct RedisStorage<'a> {
    connection: &'a mut Connection,
}

impl RedisStorage<'_> {
    pub async fn new(connection_string: String) -> RedisStorage {
        let client = redis::Client::open(connection_string).unwrap();

        let mut connection = client.get_async_connection().await.unwrap();

        RedisStorage {
            connection: &mut connection,
        }
    }
}

#[async_trait(?Send)]
impl ProofStorage for RedisStorage<'_> {
    async fn get_proof(&mut self, identifier: String) -> Result<Vec<u8>> {
        let result: Vec<u8> = self.connection.get(&identifier).await?;

        Ok(result)
    }

    async fn set_proof(&mut self, identifier: String, proof: &[u8]) -> Result<()> {
        self.connection.set(&identifier, proof).await?;

        Ok(())
    }
}
