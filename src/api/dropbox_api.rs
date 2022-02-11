use reqwest::{
    header::{HeaderMap, HeaderName, HeaderValue, CONTENT_TYPE},
    Body,
};
use std::env;
use tokio::fs::File;
use tokio_util::codec::{BytesCodec, FramedRead};

fn file_to_body(file: File) -> Body {
    let stream = FramedRead::new(file, BytesCodec::new());
    let body = Body::wrap_stream(stream);

    body
}

pub async fn upload_file(file_name: String) -> Result<reqwest::StatusCode, reqwest::Error> {
    let mut headers = HeaderMap::new();
    headers.insert(
        CONTENT_TYPE,
        HeaderValue::from_static("application/octet-stream"),
    );
    headers.insert(HeaderName::from_static("dropbox-api-arg"), HeaderValue::from_str(
		format!("{{\"path\": \"/{}\",\"mode\": \"overwrite\",\"autorename\": true,\"mute\": false,\"strict_conflict\": false}}", file_name).as_str()
	).unwrap());

    let file = File::open(file_name).await.unwrap();
    let client = reqwest::Client::new();
    let res = client
        .post("https://content.dropboxapi.com/2/files/upload")
        .headers(headers)
        .bearer_auth(env::var("DROPBOX_TOKEN").unwrap())
        .body(file_to_body(file))
        .send()
        .await?;

    println!("{:?}", res.status());

    Ok(res.status())
}

pub async fn download_file(file_name: String) -> Result<Option<String>, reqwest::Error> {
    let client = reqwest::Client::new();
    let res = client
        .post("https://content.dropboxapi.com/2/files/download")
        .bearer_auth(env::var("DROPBOX_TOKEN").unwrap())
        .header(
            HeaderName::from_static("dropbox-api-arg"),
            HeaderValue::from_str(format!("{{\"path\": \"/{}\"}}", file_name).as_str()).unwrap(),
        )
        .send()
        .await?;

    if res.status() == reqwest::StatusCode::OK {
        return Ok(Some(res.text().await.unwrap()));
    } else {
        return Ok(None);
    }
}

#[cfg(test)]
mod tests {
    use crate::consts::OLDBINUSMAYA_USER_FILE;

    use super::*;
    #[tokio::test]
    async fn upload_file_test() {
        let res = upload_file(String::from("old_binusmaya_user_data.csv")).await.unwrap();

        assert_eq!(res, reqwest::StatusCode::OK);
    }

    #[tokio::test]
    async fn download_file_test() {
        let res = download_file(String::from(OLDBINUSMAYA_USER_FILE.to_string())).await.unwrap();

        println!("{}", res.clone().unwrap());
        assert_eq!(res.unwrap().is_empty(), false);
    }
}
