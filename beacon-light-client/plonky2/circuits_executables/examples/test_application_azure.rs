use azure_storage::prelude::*;
use azure_storage_blobs::prelude::*;
use futures::stream::StreamExt;
use std::{env, fs};

#[tokio::main]
async fn main() -> azure_core::Result<()> {
    let file_name = "0.plonky2_circuit";
    let result = fs::read(file_name).unwrap();

    dotenv::dotenv().ok();

    // First we retrieve the account name and access key from environment variables.
    let account = env::var("STORAGE_ACCOUNT").expect("missing STORAGE_ACCOUNT");
    let access_key = env::var("STORAGE_ACCESS_KEY").expect("missing STORAGE_ACCOUNT_KEY");
    let container = env::var("STORAGE_CONTAINER").expect("missing STORAGE_CONTAINER");
    let blob_name = file_name;

    let storage_credentials = StorageCredentials::access_key(account.clone(), access_key);
    let blob_client =
        ClientBuilder::new(account, storage_credentials).blob_client(&container, blob_name);

    blob_client
        .put_block_blob(result)
        .content_type("application/octet-stream")
        .await?;

    let result = blob_client
        .get()
        .into_stream()
        .next()
        .await
        .unwrap()
        .unwrap();

    let result = result.data.collect().await.unwrap().to_vec();

    println!("result: {:?}", result);

    Ok(())
}
