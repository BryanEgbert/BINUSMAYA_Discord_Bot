use thirtyfour::{prelude::*, extensions::chrome::ChromeDevTools};
use std::time::Duration;
use tokio;
use serde_json::json;
use serde::Deserialize;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE, REFERER, ORIGIN, USER_AGENT, HOST, AUTHORIZATION};

pub struct BrowserMobProxy {
	pub host: &'static str,
	pub port: u16,
	pub path: &'static str
}

pub struct Selenium {
	pub driver: WebDriver,
	pub email: String,
	pub password: String
}

pub struct BinusmayaAPI {
	pub token: String
}

#[derive(Debug)]
pub enum Status {
	VALID,
	INVALID
}

#[derive(Deserialize)]
struct Port {
	port: u32
}

#[derive(Deserialize)]
struct ProxyList {
	proxyList: Vec<Port>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct RoleCategory {
	name: String,
	user_code: String,
	role_id: String,
	role_type: String,
	role_organization_id: String,
	academic_career_id: String,
	academic_career: String,
	academic_career_desc: String,
	institution_id: Option<String>,
	institution: String,
	institution_desc: String,
	academic_program: String,
	academic_program_desc: String,
	is_primary: bool,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct RoleCategories {
	name: String,
	roles: Vec<RoleCategory>
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UserProfile {
	user_id: String,
	full_name: String,
	person_code: String,
	email: String,
	user_picture_url: String,
	xP_point: f32,
	category_list: Vec<String>,
	role_categories: Vec<RoleCategories>,
}

impl BrowserMobProxy {
	pub async fn create_proxy(&self) -> Result<reqwest::StatusCode, reqwest::Error> {
		let client = reqwest::Client::new();
		let res = client.post(format!("http://{}:{}/proxy", self.host, self.port))
			.query(&[("port", "8083"), ("trustAllServers", "true")])
			.send()
			.await?;

		Ok(res.status())
	}

	pub async fn get_proxy(&self) -> Result<String, reqwest::Error> {
		let res: ProxyList = reqwest::get(format!("http://{}:{}/proxy", self.host, self.port)).await?
			.json().await?;
		
		Ok(res.proxyList[0].port.to_string())

	}

	pub async fn new_har(&self) -> Result<reqwest::StatusCode, reqwest::Error> {
		let client = reqwest::Client::new();
		let proxy = self.get_proxy().await?;

		let res = client.put(format!("http://{}:{}/proxy/{}/har", self.host, self.port, proxy))
			.query(&[("captureHeaders", "true"), ("initialPageTitle", "newbinusmaya")])
			.send().await?;

		Ok(res.status())
	}

	pub async fn get_har(&self) -> Result<serde_json::Value, reqwest::Error> {
		let proxy = self.get_proxy().await?;
		let res = reqwest::get(format!("http://{}:{}/proxy/{}/har", self.host, self.port, proxy)).await?.json().await?;

		Ok(res)
	}
}

impl Selenium {
	async fn microsoft_login(&self) -> WebDriverResult<Status> {
		let mcr_login_btn = self.driver.find_element(By::ClassName("button--azure--ad")).await?; 
		mcr_login_btn.click().await?;

		let mcr_email = self.driver.find_element(By::Id("i0116")).await?;
		tokio::time::sleep(Duration::from_millis(1000)).await;
		
		mcr_email.send_keys(TypingData::from(self.email.clone()) + Keys::Enter).await?;
		tokio::time::sleep(Duration::from_millis(1000)).await;

		self.driver.find_element(By::Id("i0118")).await?.send_keys(TypingData::from(self.password.clone()) + Keys::Enter).await?;
		self.driver.find_element(By::Id("idSIButton9")).await?.click().await?;
		tokio::time::sleep(Duration::from_millis(1000)).await;

		if self.driver.current_url().await?.contains("https://login.microsoftonline.com/") {
			Ok(Status::INVALID)
		} else {
			Ok(Status::VALID)
		}
	}
	
	pub fn init(driver: WebDriver, email: String, password: String) -> Self {
		Self {
			driver,
			email,
			password
		}
	}

	pub async fn setup(&self) -> WebDriverResult<()> {
		self.driver.set_implicit_wait_timeout(Duration::new(30, 0)).await?;

		Ok(())
	}
	
	pub async fn run(&self) -> WebDriverResult<Status> {
		let dev_tools = ChromeDevTools::new(self.driver.session());
		dev_tools.execute_cdp("Network.enable").await?;
		dev_tools.execute_cdp_with_params(
			"Network.setBlockedURLs", 
			json!({"urls": vec!["*.jpg", "*.woff2", "*.woff", "*.ttf", "*.svg", "*.jpeg", "*.png", "*.dahsboard", "*func-bm7-notification-prod*", "*.ico", "*.json", "*image/*", "*func-bm7-organization-prod*", "*ToDo*", "*func-bm7-forum-prod*", "*fonts.googleapis.com*", 
			"https://apim-bm7-prod.azure-api.net/func-bm7-course-prod/ClassSession/Ongoing/student", "https://apim-bm7-prod.azure-api.net/func-bm7-course-prod/Session/AcademicPeriod/2110"]
		})).await?;

		self.driver.get("https://newbinusmaya.binus.ac.id").await?;
	
		let status = Selenium::microsoft_login(&self).await?;

		tokio::time::sleep(Duration::from_millis(5000)).await;

		Ok(status)
	}
	
	pub async fn quit(self) -> WebDriverResult<()> {
		self.driver.quit().await?;

		Ok(())
	}
	
}

impl BinusmayaAPI {
	fn init_user_profile_header(&self) -> HeaderMap {
		let mut headers = HeaderMap::new();
		headers.insert(USER_AGENT, HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/95.0.4638.69 Safari/537.36 OPR/81.0.4196.61"));
		headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
		headers.insert(HOST, HeaderValue::from_static("apim-bm7-prod.azure-api.net"));
		headers.insert(ORIGIN, HeaderValue::from_static("https://newbinusmaya.binus.ac.id"));
		headers.insert(REFERER, HeaderValue::from_static("https://newbinusmaya.binus.ac.id/"));
		headers.insert(AUTHORIZATION, HeaderValue::from_str(&self.token[1..self.token.len()-1]).unwrap()); // string slice remove double quotes

		headers
	}

	pub async fn get_user_profile(&self) -> Result<UserProfile, reqwest::Error> {
		let client = reqwest::Client::new();
		let user_profile = client
			.get("https://apim-bm7-prod.azure-api.net/func-bm7-profile-prod/UserProfile")
			.headers(BinusmayaAPI::init_user_profile_header(self))
			.send().await?.json::<UserProfile>().await?;

		Ok(user_profile)
	}
}