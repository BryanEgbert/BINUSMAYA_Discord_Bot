use chrono::NaiveDateTime;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue, CONTENT_TYPE, REFERER, ORIGIN, USER_AGENT, HOST, AUTHORIZATION};
use serde::{Deserialize, Serialize};
use std::fmt;


#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RoleCategory {
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

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RoleActivity {
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
	is_active: bool
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SchedulePayload {
	role_activity: Vec<RoleActivity>
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RoleCategories {
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

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CustomParam {
	class_id: String,
	class_session_id: String,
	session_number: u32,
	class_session_content_id: Option<String>
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Class {
	pub date_start: String,
	pub date_end: String,
	pub title: String,
	pub content: String,
	pub location: Option<String>,
	pub location_value: Option<String>,
	pub schedule_type: String,
	pub custom_param: CustomParam,
	pub class_delivery_mode: String,
	pub delivery_mode: String,
	pub delivery_mode_desc: String,
	pub academic_career_desc: String,
	pub institution_desc: String,
	pub organization_role_id: String
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Schedule {
	#[serde(rename(deserialize = "Schedule"))]
	pub schedule: Vec<Class>,
	pub date_start: String
}

impl fmt::Display for Schedule {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		for class in &self.schedule {
			write!(f, "> Class Title: **{}**\n> Subject: **{}**\n> Start : **{}**\n> End: **{}**\n> Session: **{}**\n> Class Mode: **{}**\n\n", 
				class.title, class.content.clone(), NaiveDateTime::parse_from_str(class.date_start.as_str(), "%FT%X").unwrap(), NaiveDateTime::parse_from_str(class.date_end.as_str(), "%FT%X").unwrap(), class.custom_param.session_number, class.class_delivery_mode)?;
		}

		Ok(())
	}
}
pub struct BinusmayaAPI {
	pub token: String
}

impl RoleActivity {
	fn new(role_category: RoleCategory) -> Self {
		RoleActivity {
			name: role_category.name,
			user_code: role_category.user_code,
			role_id: role_category.role_id,
			role_type: role_category.role_type,
			role_organization_id: role_category.role_organization_id,
			academic_career_id: role_category.academic_career_id,
			academic_career: role_category.academic_career,
			academic_career_desc: role_category.academic_career_desc,
			institution_id: role_category.institution_id,
			institution: role_category.institution,
			institution_desc: role_category.institution_desc,
			academic_program: role_category.academic_program,
			academic_program_desc: role_category.academic_program_desc,
			is_primary: role_category.is_primary,
			is_active: true
		}
	}
}

impl BinusmayaAPI {
	fn init_user_profile_header(&self) -> HeaderMap {
		let mut headers = HeaderMap::new();
		headers.insert(USER_AGENT, HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/95.0.4638.69 Safari/537.36 OPR/81.0.4196.61"));
		headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
		headers.insert(ORIGIN, HeaderValue::from_static("https://newbinusmaya.binus.ac.id"));
		headers.insert(HOST, HeaderValue::from_static("apim-bm7-prod.azure-api.net"));
		headers.insert(REFERER, HeaderValue::from_static("https://newbinusmaya.binus.ac.id/"));
		headers.insert(AUTHORIZATION, HeaderValue::from_str(&self.token).unwrap()); // string slice remove double quotes

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

	pub async fn get_schedule(&self) -> Result<Schedule, reqwest::Error> {
		let user_profile: UserProfile = BinusmayaAPI::get_user_profile(self).await.expect("Error in getting user profile");

		let mut headers = HeaderMap::new();
		headers.extend(BinusmayaAPI::init_user_profile_header(self));
		headers.insert(HeaderName::from_static("roid"), HeaderValue::from_str(user_profile.role_categories[0].roles[0].role_organization_id.as_str()).unwrap());
		headers.insert(HeaderName::from_static("roleid"), HeaderValue::from_str(user_profile.role_categories[0].roles[0].role_id.as_str()).unwrap());
		headers.insert(HeaderName::from_static("rolename"), HeaderValue::from_static("Student"));
		headers.insert(HeaderName::from_static("instituion"), HeaderValue::from_static("BNS01"));
		headers.insert(HOST, HeaderValue::from_static("func-bm7-schedule-prod.azurewebsites.net"));
		headers.insert(HeaderName::from_static("academiccareer"), HeaderValue::from_static("RS1"));

		let role_activity = RoleActivity::new(user_profile.role_categories[0].roles[0].clone());
		let mut role_activities = Vec::new();
		role_activities.push(role_activity);


		let client = reqwest::Client::new();
		let schedule = client
			.post(format!("https://func-bm7-schedule-prod.azurewebsites.net/api/Schedule/Date-v1/{}", chrono::offset::Utc::now().format("%Y-%-m-%-d")))
			.headers(headers)
			.json(&SchedulePayload {
				role_activity: role_activities
			})
			.send()
			.await.expect("error when serializing")
			.json::<Schedule>().await.expect("Something's wrong when parsing response");

		Ok(schedule)
	}
}