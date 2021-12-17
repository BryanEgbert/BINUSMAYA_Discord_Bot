use std::process;
use thirtyfour::prelude::*;
use std::time::Duration;
use tokio;

pub struct BrowserMobProxy {
	pub host: &'static str,
	pub port: u16,
	pub path: &'static str
}

pub struct Selenium {
	pub driver: WebDriver
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

	pub async fn create_proxy(&self) -> Result<reqwest::StatusCode, reqwest::Error> {
		let client = reqwest::Client::new();
		let res = client.post(format!("http://{}:{}/proxy", self.host, self.port))
			.query(&[("port", "8082"), ("bindAddress", self.host), ("serverBindAddress", self.host), ("trustAllServers", "true")])
			.send()
			.await?;

		Ok(res.status())
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

		let res = client.put(format!("http://{}:{}/proxy/{}/har", self.host, self.port, proxy))
			.query(&[("captureHeaders", "true"), ("initialPageTitle", "newbinusmaya")])
			.send().await?;

		Ok(res.status())
	}

	pub async fn get_har(&self) -> Result<String, reqwest::Error> {
		let proxy = self.get_proxy().await?;
		let res = reqwest::get(format!("http://{}:{}/proxy/{}/har", self.host, self.port, proxy)).await?.text().await?;

		Ok(res.to_string())
	}

	pub async fn close(&self) -> Result<reqwest::StatusCode, reqwest::Error> {
		let client = reqwest::Client::new();
		let port = self.get_proxy().await?;

		let res = client.delete(format!("http://{}:{}/proxy/{}", self.host, self.port, port)).send().await?;
		
		Ok(res.status())
	}
}

impl Selenium {
	async fn microsoft_login(&self) -> WebDriverResult<()> {
		let mcr_login_btn = self.driver.find_element(By::ClassName("button--azure--ad")).await?; 
		mcr_login_btn.click().await?;

		let mcr_email = self.driver.find_element(By::Id("i0116")).await?;
		tokio::time::sleep(Duration::from_millis(1000)).await;
		
		mcr_email.send_keys(TypingData::from("bryan.egbert@binus.ac.id") + Keys::Enter).await?;
		tokio::time::sleep(Duration::from_millis(1000)).await;

		// self.driver.find_element(By::Id("idSIButton9")).await?.click().await?;
		tokio::time::sleep(Duration::from_millis(1000)).await;

		self.driver.find_element(By::Id("i0118")).await?.send_keys("ControlTheBoard90").await?;
		self.driver.find_element(By::Id("idSIButton9")).await?.click().await?;

		self.driver.find_element(By::Id("idSIButton9")).await?.click().await?;
		

		Ok(())
	}

	pub fn init(driver: WebDriver) -> Self {
		Self {
			driver
		}
	}

	pub async fn setup(&self) -> WebDriverResult<()> {

		

		self.driver.set_implicit_wait_timeout(Duration::new(30, 0)).await?;

		Ok(())

	}
	
	pub async fn run(&self) -> WebDriverResult<()> {
		// self.driver.get("https://www.google.com").await?;
		self.driver.get("https://newbinusmaya.binus.ac.id").await?;
	
		Selenium::microsoft_login(&self).await?;
	
		tokio::time::sleep(Duration::from_millis(10000)).await;
	
		
		Ok(())
	}
	
	pub async fn quit(self) -> WebDriverResult<()> {
		self.driver.quit().await?;

		Ok(())
	}
	
}

