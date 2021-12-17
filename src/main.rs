#![allow(non_snake_case)]
mod prelude;

use thirtyfour::Capabilities;
use thirtyfour::prelude::*;
use prelude::*;
use thirtyfour::common::capabilities::desiredcapabilities::Proxy; 
use std::time::Duration;

#[tokio::main]
async fn main() -> WebDriverResult<()> {
    let proxy = BrowserMobProxy {host: "localhost", port: 8081, path: "./browsermob-proxy-2.1.4/bin/browsermob-proxy"};

    
    // BrowserMobProxy::run(&proxy);
    
    let res = proxy.create_proxy().await?;
    println!("create proxy: {}", res);
    let proxy_list = proxy.get_proxy().await?;
    let proxy_port = 8082;
    println!("{}", proxy_list);
    
    let mut caps = DesiredCapabilities::edge();
    // caps.set_proxy(Proxy::AutoConfig {url: format!("http://{}:{}", proxy.host, 8082)})?;
    caps.set_proxy(Proxy::Manual {
			ftp_proxy: None, 
			http_proxy: Some(format!("http://{}:{}", proxy.host, proxy_port)), 
			ssl_proxy: None,
			socks_proxy: None,
			socks_version: None,
			socks_username: None,
			socks_password: None,
			no_proxy: None
		})?;
	caps.accept_ssl_certs(true)?;

    println!("{}", caps.get_mut());
    let selenium = Selenium::init(WebDriver::new("http://localhost:4445/wd/hub", &caps).await?);

    selenium.setup().await?;

    
    
    BrowserMobProxy::new_har(&proxy).await?;

    selenium.run().await?;
    tokio::time::sleep(Duration::from_millis(2000)).await;

    let har: String = BrowserMobProxy::get_har(&proxy).await?;

    println!("{}", har);

    selenium.quit().await?;

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
