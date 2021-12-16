#![allow(non_snake_case)]
mod prelude;

use thirtyfour::prelude::WebDriverResult;
use prelude::*;


#[tokio::main]
async fn main() -> WebDriverResult<()> {
    let proxy = BrowserMobProxy {host: "localhost", port: 8081, path: "./browsermob-proxy-2.1.4/bin/browsermob-proxy"};

    // BrowserMobProxy::run(&proxy);
    
    BrowserMobProxy::create_proxy(&proxy).await?;
    let proxy_list = BrowserMobProxy::get_proxy(&proxy).await?;
    println!("{}", proxy_list);

    let selenium = Selenium {url: "http://localhost:4445/wd/hub".to_string()};

    selenium.run(&proxy).await?;
    // BrowserMobProxy::new_har(&proxy).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn create_proxy() -> Result<(), reqwest::Error> {
        let proxy = BrowserMobProxy {host: "localhost", port: 8081, path: "./browsermob-proxy-2.1.4/bin/browsermob-proxy"};

        BrowserMobProxy::create_proxy(&proxy).await?; 
        let proxy_list = BrowserMobProxy::get_proxy(&proxy).await?;

        assert_eq!(proxy_list.contains("8"), true);

        BrowserMobProxy::close(&proxy).await?;
        let proxy_list = BrowserMobProxy::get_proxy(&proxy).await?;

        assert_eq!(proxy_list, "null");

        Ok(())
    }
}
