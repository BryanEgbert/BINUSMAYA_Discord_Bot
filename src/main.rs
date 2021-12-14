#![allow(non_snake_case)]
mod init;

pub use crate::init::proxy::Proxy;

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let proxy = Proxy {host: "localhost", port: 8081, path: "./browsermob-proxy-2.1.4/bin/browsermob-proxy"};

    // Proxy::run(&proxy);
    
    // Proxy::create_proxy(&proxy).await?;
    let proxy_list = Proxy::get_proxy(&proxy).await?;
    
    Proxy::close(&proxy).await?;


    Ok(())
}
