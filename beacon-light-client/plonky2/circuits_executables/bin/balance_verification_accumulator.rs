use anyhow::Result;
use futures_lite::future;


// TODO: full setup
/*
    - How will it work?
    -
 */
fn main() -> Result<()> {
    future::block_on(async_main())
}


async fn async_main() -> Result<()> {
    Ok(())
}
