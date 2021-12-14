use std::process;

pub struct Proxy {
	pub host: &'static str,
	pub port: u16,
	pub path: &'static str
}

impl Proxy {
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

	pub async fn new_har(&self) -> Result<(), reqwest::Error> {
		let client = reqwest::Client::new();
		let proxy = self.get_proxy().await?;

		if(format!("{}", proxy) != "null") {
			client.put(format!("http://{}:{}/proxy/{}/har", self.host, self.port, proxy)).send().await?;
		}
		

		Ok(())
	} 

	pub async fn close(&self) -> Result<(), reqwest::Error> {
		let client = reqwest::Client::new();
		let port = self.get_proxy().await?;

		client.delete(format!("http://{}:{}/proxy/{}", self.host, self.port, port)).send().await?;
		
		Ok(())
	}


}