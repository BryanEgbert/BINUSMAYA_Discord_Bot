#![allow(non_snake_case)]
mod prelude;

use std::time::Duration;
use thirtyfour::Capabilities;
use thirtyfour::prelude::*;
use prelude::*;
use thirtyfour::common::capabilities::desiredcapabilities::Proxy; 
use std::{fs, io::prelude::*};


#[tokio::main]
async fn main() -> WebDriverResult<()> {
    let proxy = BrowserMobProxy {host: "localhost", port: 8082, path: "./browsermob-proxy-2.1.4/bin/browsermob-proxy"};

    
    // BrowserMobProxy::run(&proxy);
    
    proxy.create_proxy().await?;

    let proxy_list = proxy.get_proxy().await?;
    println!("{}", proxy_list);

    let proxy_port = 8083;
    
    let mut caps = DesiredCapabilities::chrome();
    caps.set_proxy(Proxy::Manual {
        ftp_proxy: None, 
        http_proxy: Some(format!("http://{}:{}", proxy.host, proxy_port)), 
        ssl_proxy: Some(format!("http://{}:{}", proxy.host, proxy_port)),
        socks_proxy: None,
        socks_version: None,
        socks_username: None,
        socks_password: None,
        no_proxy: None
    })?;
    caps.accept_ssl_certs(true)?;
    caps.set_binary("/usr/bin/google-chrome")?;
    caps.add_chrome_arg("--proxy-server=http://localhost:8083")?;
    caps.add_chrome_arg("--ignore-certificate-errors")?;
    caps.set_headless()?;
    
    let selenium = Selenium::init(WebDriver::new("http://localhost:4444", &caps).await?);

    selenium.setup().await?;

    
    BrowserMobProxy::new_har(&proxy).await?;
    selenium.run().await?;
    let har = BrowserMobProxy::get_har(&proxy).await?;
    let len = har["log"]["entries"].as_array().unwrap().len();

    println!("{:?}", len);
    println!("{}", har["log"]["entries"][len - 1]["request"]["headers"][6]["value"]);

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
