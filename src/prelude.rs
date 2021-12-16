use std::process;
use thirtyfour::prelude::*;
use thirtyfour::Capabilities;
use thirtyfour::common::capabilities::desiredcapabilities::Proxy; 

pub struct BrowserMobProxy {
	pub host: &'static str,
	pub port: u16,
	pub path: &'static str
}

pub struct Selenium {
	pub url: String
}

impl BrowserMobProxy {
	pub fn run(&self){
		let host = self.host;
		let port = self.port.to_string();

		if cfg!(target_os = "windows") {
			process::Command::new(self.path).args(["--address", host, "--port", port.as_str()]).spawn().expect("failed to run proxy");
		} else {
			process::Command::new("sh").args([self.path, "--address", host, "--port", port.as_str()]).spawn().expect("failed to run proxy");
		}

	}

	pub async fn create_proxy(&self) -> Result<(), reqwest::Error> {

		let client = reqwest::Client::new();
		client.post(format!("http://{}:{}/proxy", self.host, self.port)).send().await?;
	
		Ok(())
	}

	pub async fn get_proxy(&self) -> Result<String, reqwest::Error> {
		let res = reqwest::get(format!("http://{}:{}/proxy", self.host, self.port)).await?
			.text().await?;
		let parsed_res = &json::parse(&res).unwrap()["proxyList"][0];
		
		Ok(parsed_res["port"].to_string())

	}

	pub async fn new_har(&self) -> Result<reqwest::StatusCode, reqwest::Error> {
		let client = reqwest::Client::new();
		let proxy = self.get_proxy().await?;

		let res = client.put(format!("http://{}:{}/proxy/{}/har", self.host, self.port, proxy)).send().await?;

		Ok(res.status())
	}

	pub async fn get_har(&self) -> Result<reqwest::StatusCode, reqwest::Error> {
		let proxy = self.get_proxy().await?;
		let res = reqwest::get(format!("http://{}:{}/proxy/{}/har", self.host, self.port, proxy)).await?;

		Ok(res.status())
	}

	pub async fn close(&self) -> Result<reqwest::StatusCode, reqwest::Error> {
		let client = reqwest::Client::new();
		let port = self.get_proxy().await?;

		let res = client.delete(format!("http://{}:{}/proxy/{}", self.host, self.port, port)).send().await?;
		
		Ok(res.status())
	}
}

impl Selenium {
	pub async fn run(&self, proxy: &BrowserMobProxy) -> WebDriverResult<()> {
		let proxy_port = BrowserMobProxy::get_proxy(&proxy).await?;

		let caps = DesiredCapabilities::edge().set_proxy(Proxy::AutoConfig {url: format!("http://localhost:{}", proxy_port)})?;
		let driver = WebDriver::new(self.url.as_str(), &caps).await?;

		driver.get("https://newbinusmaya.binus.ac.id").await?;

		driver.quit().await?;

		Ok(())
	}
}

