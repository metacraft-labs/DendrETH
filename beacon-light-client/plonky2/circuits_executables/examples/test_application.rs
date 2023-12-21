use std::env;

use aws_config::{meta::region::RegionProviderChain, Region};
use aws_sdk_s3::{primitives::ByteStream, Client, Error};

#[tokio::main]
async fn main() -> Result<(), Error> {
    env::set_var("AWS_ACCESS_KEY_ID", "test");
    env::set_var("AWS_SECRET_ACCESS_KEY", "test");

    let config = aws_config::from_env()
        .region(Region::new("us-west-2"))
        .endpoint_url("http://localhost:4566")
        .load()
        .await;

    let client = Client::new(&config);

    let resp = client.list_buckets().send().await?;
    println!("Buckets:");

    for bucket in resp.buckets() {
        println!("{}", bucket.name().unwrap_or_default());
    }

    let bucket = "your-bucket-name";
    let key = "your-object-key";

    let data: Vec<u8> = vec![13, 123, 23];

    let byte_stream = ByteStream::from(data);

    // Put object
    client
        .put_object()
        .bucket(bucket)
        .key(key)
        .body(byte_stream)
        .send()
        .await?;

    Ok(())
}
