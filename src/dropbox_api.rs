use reqwest::{header::{HeaderMap, HeaderValue, HeaderName, CONTENT_TYPE}, Body};
use std::env;
use tokio::fs::File;
use tokio_util::codec::{BytesCodec, FramedRead};
pub struct DropboxAPI {
	pub file_path: String,
}

impl DropboxAPI {
	fn file_to_body(&self, file: File) -> Body {
		let stream = FramedRead::new(file, BytesCodec::new());
		let body = Body::wrap_stream(stream);

		body
	}

	async fn upload_file(&self) -> Result<reqwest::StatusCode, reqwest::Error> {
		let mut headers = HeaderMap::new();
		headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/octet-stream"));
		headers.insert(HeaderName::from_static("dropbox-api-arg"), HeaderValue::from_static(
			"{\"path\": \"/user_data.csv\",\"mode\": \"overwrite\",\"autorename\": true,\"mute\": false,\"strict_conflict\": false}"
		));

		let file = File::open(self.file_path.clone()).await.unwrap();
		let client = reqwest::Client::new();
		let res = client
			.post("https://content.dropboxapi.com/2/files/upload")
			.headers(headers)
			.bearer_auth(env::var("DROPBOX_TOKEN").unwrap())
			.body(self.file_to_body(file))
			.send().await?;

		println!("{:?}", res.status());

		Ok(res.status())
	}

	async fn download_file(&self) -> Result<String, reqwest::Error> {
		let client = reqwest::Client::new();
		let res = client
			.post("https://content.dropboxapi.com/2/files/download")
			.bearer_auth(env::var("DROPBOX_TOKEN").unwrap())
			.header(HeaderName::from_static("dropbox-api-arg"), HeaderValue::from_static("{\"path\": \"/user_data.csv\"}"))
			.send().await?;


		Ok(res.text().await.unwrap())
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	#[tokio::test]
	async fn upload_file_test() {
		let dropbox = DropboxAPI {file_path: String::from("user_data.csv")};
		let res = dropbox.upload_file().await.unwrap();

		assert_eq!(res, reqwest::StatusCode::OK);
	}

	#[tokio::test]
	async fn download_file_test() {
		let dropbox = DropboxAPI {file_path: String::from("user_data.csv")};
		let res = dropbox.download_file().await.unwrap();

		println!("{}", res);
		assert_eq!(res.is_empty(), false);
	}
}