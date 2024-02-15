use anyhow::Result;
use futures_lite::future;


fn main() -> Result<()> {
    future::block_on(async_main())
}


async fn async_main() -> Result<()> {

    Ok(())
}
