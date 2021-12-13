pub mod proxy {
	pub async fn initProxy() -> Result<(), reqwest::Error> {
		let client = reqwest::Client::new();
		client.post("http://localhost:8080/proxy").send().await?;
		
		Ok(())
	}

	pub async fn getProxy() -> Result<(), reqwest::Error> {
		let res = reqwest::get("http://localhost:8080/proxy").await?;
		let content = res.text().await?;

		println!("{}", content);

		Ok(())
	}
}