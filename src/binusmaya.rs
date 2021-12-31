use chrono::NaiveDateTime;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue, CONTENT_TYPE, REFERER, ORIGIN, USER_AGENT, HOST, AUTHORIZATION};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AcademicPeriod {
	academic_period: String,
	academic_period_description: String,
	academic_period_id: Option<String>,
	academic_period_status: bool,
	is_running_period: bool,
	term_begin_date: String,
	term_end_date: String
}

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
	pub class_id: String,
	pub class_session_id: String,
	pub session_number: u32,
	pub class_session_content_id: Option<String>
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ScheduleDetails {
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
	pub schedule: Vec<ScheduleDetails>,
	pub date_start: String
}

impl fmt::Display for Schedule {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		for class in &self.schedule {
			write!(f, "> Class Title: **{}**\n> Subject: **{}**\n> Start : **{}**\n> End: **{}**\n> Session: **{}**\n> Class Delivery Mode: **{}**\n\n", 
				class.title, class.content.clone(), 
				NaiveDateTime::parse_from_str(class.date_start.as_str(), "%FT%X").unwrap(), 
				NaiveDateTime::parse_from_str(class.date_end.as_str(), "%FT%X").unwrap(), 
				class.custom_param.session_number, 
				class.class_delivery_mode
			)?;
		}

		Ok(())
	}
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ClassSessionProgress {
	completed: u8,
	in_progress: u8,
	not_started: u8
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Lecturer {
	id: String,
	name: String,
	picture_url: String,
	role: String,
	user_code: String
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Resource {
	pub android_redirect_url: Option<String>,
	pub assessment_type: Option<String>,
	pub due_date: String,
	pub duration: String,
	pub id: String,
	pub index: String,
	pub ios_redirect_url: Option<String>,
	pub is_open: bool,
	pub is_overdue: bool,
	pub last_updated_date: String,
	pub name: String,
	pub progress_stamp: u8,
	pub progress_status: u8,
	pub resource_last_updated_date: String,
	pub resource_status: String,
	pub resource_type: String,
	pub thumbnail: Option<String>,
	pub times_accessed: u8,
	pub token: Option<String>,
	#[serde(rename(deserialize = "type"))]
	pub material_type: Option<String>,
	pub url: Option<String>
}

#[derive(Deserialize, Debug)]
#[serde(transparent)]
pub struct ResourceList {
	resources: Vec<Resource>
}

impl fmt::Display for ResourceList {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		for resource in &self.resources.clone() {
			write!(f, "> Name: **{}**\n> Duration: **{} min**\n> Type: **{}**\n\n", 
				resource.name, 
				resource.duration.parse::<u32>().unwrap() / 60,
				resource.resource_type
			)?;
		}

		Ok(())
	}
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SessionDetails {
	pub class_delivery_mode: String,
	pub class_session_progress: ClassSessionProgress,
	pub course_sub_topic: Vec<String>,
	pub date_end: String,
	pub date_start: String,
	pub delivery_mode: String,
	pub delivery_mode_desc: String,
	pub end_date_session_utc: String,
	pub is_ended: bool,
	pub join_url: Option<String>,
	pub lecturers: Vec<Lecturer>,
	pub meeting_end: String,
	pub meeting_start: String,
	pub resources: ResourceList,
	pub session_number: u8,
	pub start_date_session_utc: String,
	pub status: Option<String>,
	pub topic: String,
	pub total_resource: u8
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Class {
	pub class_Code: String,
	pub class_id: String,
	pub course_code: String,
	pub course_name: String,
	pub ssr_component: String
}

#[derive(Deserialize, Debug, Clone)]
#[serde(transparent)]
pub struct ClassVec {
	pub classes: Vec<Class>
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Session {
	pub class_session_number: String,
	pub course_topic_id: String,
	pub id: String,
	pub status: u8
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClassDetails {
	pub academic_career: String,
	pub academic_career_id: String,
	pub academic_period: String,
	pub academic_period_id: String,
	pub class_code: String,
	pub class_group_id: Option<String>,
	pub course_code: String,
	pub course_id: String,
	pub course_title_en: String,
	pub crse_id: String,
	pub institution: String,
	pub lecturers: Vec<Lecturer>,
	pub n_sksp: String,
	pub n_skst: String,
	pub revision: u8,
	pub sessions: Vec<Session>,
	pub ssr_component: String

}

impl fmt::Display for ClassVec {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		for class in  &self.classes {
			write!(f, "> Class code: **{}**\n> Course code: **{}**\n> Course name: **{}**\n> Class component: **{}**\n\n", 
				class.class_Code, class.course_code, class.course_name, class.ssr_component)?;
		}
		
		Ok(())
	}
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StudentProgressPayload {
	resource_id: String,
	status: u8
}

#[derive(Clone)]
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

	async fn init_full_header(&self, user_profile: &UserProfile) -> HeaderMap {
		let mut headers = HeaderMap::new();
		headers.extend(BinusmayaAPI::init_user_profile_header(self));
		headers.insert(HeaderName::from_static("roid"), HeaderValue::from_str(user_profile.role_categories[0].roles[0].role_organization_id.as_str()).unwrap());
		headers.insert(HeaderName::from_static("roleid"), HeaderValue::from_str(user_profile.role_categories[0].roles[0].role_id.as_str()).unwrap());
		headers.insert(HeaderName::from_static("rolename"), HeaderValue::from_static("Student"));
		headers.insert(HeaderName::from_static("instituion"), HeaderValue::from_static("BNS01"));
		headers.insert(HeaderName::from_static("academiccareer"), HeaderValue::from_static("RS1"));

		headers
	}

	async fn get_academic_period(&self) -> Result<Option<String>, reqwest::Error> {
		let user_profile: UserProfile = BinusmayaAPI::get_user_profile(self).await.expect("Error in getting user profile");
		let client = reqwest::Client::new();
		let response = client
			.get("https://apim-bm7-prod.azure-api.net/func-bm7-course-prod/AcademicPeriod/Student")
			.headers(BinusmayaAPI::init_full_header(self, &user_profile).await)
			.send().await?.json::<Vec<AcademicPeriod>>().await?;

		let mut academic_period: String = String::new();

		for i in response {
			if i.is_running_period {
				academic_period = i.academic_period;
			}
		}
		
		Ok(Some(academic_period))
	}

	pub async fn get_schedule(&self) -> Result<Option<Schedule>, reqwest::Error> {
		let user_profile: UserProfile = BinusmayaAPI::get_user_profile(self).await.expect("Error in getting user profile");
		let mut headers = HeaderMap::new();
		headers.extend(BinusmayaAPI::init_full_header(self, &user_profile).await);
		headers.insert(HOST, HeaderValue::from_static("func-bm7-schedule-prod.azurewebsites.net"));

		let role_activity = RoleActivity::new(user_profile.role_categories[0].roles[0].clone());
		let mut role_activities = Vec::new();
		role_activities.push(role_activity);


		let client = reqwest::Client::new();
		let response = client
			.post(format!("https://func-bm7-schedule-prod.azurewebsites.net/api/Schedule/Date-v1/{}", chrono::offset::Utc::now().format("%Y-%-m-%-d")))
			.headers(headers)
			.json(&SchedulePayload {
				role_activity: role_activities
			})
			.send()
			.await.expect("error when serializing");

		if response.status() != reqwest::StatusCode::NO_CONTENT {
			return Ok(Some(response.json::<Schedule>().await.expect("Something's wrong when parsing response")));
		} else {
			return Ok(None);
		}
	}

	pub async fn get_resource(&self, session_id: String) -> Result<SessionDetails, reqwest::Error> {
		let user_profile: UserProfile = BinusmayaAPI::get_user_profile(self).await.expect("Error in getting user profile");

		let mut headers = HeaderMap::new();
		headers.extend(BinusmayaAPI::init_full_header(self, &user_profile).await);

		let client = reqwest::Client::new();
		let session_details = client
			.get(format!("https://apim-bm7-prod.azure-api.net/func-bm7-course-prod/ClassSession/Session/{}/Resource/Student", session_id))
			.query(&[("isWeb", "true")])
			.headers(headers)
			.send().await.expect("Something's wrong when sending request")
			.json::<SessionDetails>().await.expect("Something's wrong when parsing response");

		Ok(session_details)
	}

	pub async fn get_classes(&self) -> Result<ClassVec, reqwest::Error> {
		let user_profile: UserProfile = BinusmayaAPI::get_user_profile(self).await.expect("Error in getting user profile");

		let mut headers = HeaderMap::new();
		headers.extend(BinusmayaAPI::init_full_header(self, &user_profile).await);

		let client = reqwest::Client::new();
		let classes = client
			.get("https://apim-bm7-prod.azure-api.net/func-bm7-course-prod/Class/Active/Student")
			.headers(headers)
			.send().await?
			.json::<ClassVec>().await.expect("Something's wrong when parsing");

		Ok(classes)
	}

	pub async fn get_class_details(&self, class_id: String) -> Result<ClassDetails, reqwest::Error> {
		let user_profile: UserProfile = BinusmayaAPI::get_user_profile(self).await.expect("Error in getting user profile");

		let mut headers = HeaderMap::new();
		headers.extend(BinusmayaAPI::init_full_header(self, &user_profile).await);

		let client = reqwest::Client::new();
		let response = client
			.get(format!("https://apim-bm7-prod.azure-api.net/func-bm7-course-prod/ClassSession/Class/{}/Student", class_id))
			.headers(headers)
			.send().await?
			.json::<ClassDetails>().await.expect("Something's wrong when parsing response");

		Ok(response)
	}

	pub async fn update_student_progress(&self, resource_id: String) -> Result<reqwest::StatusCode, reqwest::Error> {
		let user_profile: UserProfile = BinusmayaAPI::get_user_profile(self).await.expect("Error in getting user profile");

		let mut headers = HeaderMap::new();
		headers.extend(BinusmayaAPI::init_full_header(self, &user_profile).await);

		let client = reqwest::Client::new();
		let response = client
			.post("https://apim-bm7-prod.azure-api.net/func-bm7-course-prod/StudentProgress")
			.headers(headers)
			.json::<StudentProgressPayload>(&StudentProgressPayload{resource_id, status: 2})
			.send().await.expect("Something's wrong when sending request");

		Ok(response.status())
	}
}