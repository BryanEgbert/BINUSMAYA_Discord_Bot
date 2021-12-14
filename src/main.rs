#![allow(non_snake_case)]
mod proxy;

use proxy::Proxy;


#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let proxy = Proxy {host: "localhost", port: 8081, path: "./browsermob-proxy-2.1.4/bin/browsermob-proxy"};

    // Proxy::run(&proxy);
    
    // Proxy::create_proxy(&proxy).await?;
    let proxy_list = Proxy::get_proxy(&proxy).await?;

    Proxy::new_har(&proxy).await?;
    
    Proxy::close(&proxy).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn create_proxy() -> Result<(), reqwest::Error> {
        let proxy = Proxy {host: "localhost", port: 8081, path: "./browsermob-proxy-2.1.4/bin/browsermob-proxy"};

        Proxy::create_proxy(&proxy).await?; 
        let proxy_list = Proxy::get_proxy(&proxy).await?;

        assert_eq!(proxy_list.contains("8"), true);

        Proxy::close(&proxy).await?;
        let proxy_list = Proxy::get_proxy(&proxy).await?;

        assert_eq!(proxy_list, "null");

        Ok(())
    }
}
