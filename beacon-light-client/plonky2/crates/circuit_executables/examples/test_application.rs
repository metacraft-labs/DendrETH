use anyhow::Result;
use aws_config::Region;
use aws_sdk_s3::{primitives::ByteStream, Client, Config};
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    env::set_var("AWS_ACCESS_KEY_ID", "test");
    env::set_var("AWS_SECRET_ACCESS_KEY", "test");

    let aws_config = aws_config::from_env().load().await;

    let s3_config = Config::builder()
        .credentials_provider(aws_config.credentials_provider().unwrap())
        .behavior_version_latest()
        .region(Region::new("us-west-2"))
        .endpoint_url("http://localhost:4566")
        .force_path_style(true)
        .clone()
        .build();

    let client = Client::from_conf(s3_config);

    let resp = client.list_buckets().send().await?;
    println!("Buckets:");

    for bucket in resp.buckets() {
        println!("{}", bucket.name().unwrap_or_default());
    }

    let bucket_name = "your-bucket-name";

    // let resp = client.list_objects_v2().bucket(bucket_name).send().await?;

    // println!("Objects in bucket {}:", bucket_name);

    // let contents = resp.contents();

    // for object in contents {
    //     println!("Key: {}", object.key().unwrap_or_default());

    //     let key = object.key().unwrap_or_default();

    //     let resp = client
    //         .get_object()
    //         .bucket(bucket_name)
    //         .key(key)
    //         .send()
    //         .await?;

    //     let mut body = resp.body.collect().await?;
    //     let mut content = body.into_bytes().to_vec();

    //     let content_str = String::from_utf8_lossy(&content);

    //     println!("Value: {}", content_str);
    // }

    let key = "proof.bin";

    let data = "basi maikata lelelelelelel";
    let byte_stream = ByteStream::from(data.as_bytes().to_vec());

    // client.create_bucket().bucket(bucket_name).send().await?;

    // Upload the object
    client
        .put_object()
        .bucket(bucket_name)
        .key(key)
        .body(byte_stream)
        .send()
        .await
        .unwrap();

    Ok(())
}
