#![allow(non_snake_case)]
mod init;
pub use crate::init::proxy;

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    println!("Hello, world!");
    proxy::initProxy().await?;
    proxy::getProxy().await?;

    Ok(())
}
