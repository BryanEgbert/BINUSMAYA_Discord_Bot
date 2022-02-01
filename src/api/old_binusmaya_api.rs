use reqwest::header::{HeaderMap, ACCEPT, ACCEPT_ENCODING, CONTENT_TYPE, HOST, HeaderValue, ORIGIN, REFERER, COOKIE, HeaderName};
use serde::Deserialize;


#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct SATPoint {
	pub activity_type: String,
	pub points: u8,
	pub target_points: u8,
	pub total_points: u8
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct ComServ {
	community_service_type: String,
	points: u8,
	total_points: u8,
	target_points: u8
}
pub struct OldBinusmayaApi {
	pub cookie: String
}

impl OldBinusmayaApi {
	async fn init_header(&self) -> HeaderMap {
		let mut headers = HeaderMap::new();
		headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json;charset=utf-8"));
		headers.insert(HOST, HeaderValue::from_static("binusmaya.binus.ac.id"));
		headers.insert(ORIGIN, HeaderValue::from_static("https://binusmaya.binus.ac.id"));
		headers.insert(REFERER, HeaderValue::from_static("https://binusmaya.binus.ac.id/newStudent/"));
		headers.insert(COOKIE, HeaderValue::from_str(self.cookie.as_str()).unwrap());
		headers.insert(HeaderName::from_static("content-length"), HeaderValue::from_static("0"));

		headers
	}

	pub async fn get_sat(&self) -> Result<Vec<SATPoint>, reqwest::Error> {
		let headers = self.init_header().await;
		let client = reqwest::Client::new();
		let response = client
			.post("https://binusmaya.binus.ac.id/services/ci/index.php/sat/studentactivitytranscript/GetStudentActivityPoint")
			.headers(headers)
			.send()
			.await?;
		println!("{:?}", response.status());

		Ok(response
			.json::<Vec<SATPoint>>()
			.await.expect("Something's wrong when parsing"))
	}

	pub async fn get_comnunity_service(&self) -> Result<Vec<ComServ>, reqwest::Error> {
		let headers = self.init_header().await;
		let client = reqwest::Client::new();
		let response = client
			.post("https://binusmaya.binus.ac.id/services/ci/index.php/sat/studentactivitytranscript/GetCommunityServices")
			.headers(headers)
			.send()
			.await?
			.json::<Vec<ComServ>>()
			.await?;

		Ok(response)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	#[tokio::test]
	async fn get_sat() {
		let binusmaya_api = OldBinusmayaApi {
			cookie: "PHPSESSID=mc03m7k4n74e8b7fp0g29r7c16".to_string()
		};

		let res = binusmaya_api.get_sat().await.unwrap();

		println!("{:#?}", res);

		assert_eq!(res.is_empty(), false);
	}

	#[tokio::test]
	async fn get_community_service() {
		let binusmaya_api = OldBinusmayaApi {
			cookie: "PHPSESSID=mc03m7k4n74e8b7fp0g29r7c16".to_string()
		};

		let res = binusmaya_api.get_comnunity_service().await.unwrap();

		println!("{:#?}", res);

		assert_eq!(res.is_empty(), false);
	}
}