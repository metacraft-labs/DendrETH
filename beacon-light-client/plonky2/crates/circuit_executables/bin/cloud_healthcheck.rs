use anyhow::Result;
use circuit_executables::{
    crud::proof_storage::proof_storage::create_proof_storage,
    utils::{get_default_config, CommandLineOptionsBuilder},
};

#[tokio::main]
pub async fn main() -> Result<()> {
    dotenv::from_path(".env").unwrap();

    let config = get_default_config()?;

    let matches = CommandLineOptionsBuilder::new("commitment_mapper")
        .with_redis_options(&config.redis_host, config.redis_port, &config.redis_auth)
        .with_work_queue_options()
        .with_proof_storage_options()
        .get_matches();

    let mut proof_storage = create_proof_storage(&matches).await;

    let key = "asd".to_string();
    proof_storage.set_proof(key.clone(), &[0, 1, 2]).await?;

    let result = proof_storage.get_proof(key.clone()).await?;

    println!("{result:?}");

    Ok(())
}
