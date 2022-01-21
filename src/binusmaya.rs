use chrono::{NaiveDateTime, NaiveDate, Local, TimeZone};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue, CONTENT_TYPE, REFERER, ORIGIN, USER_AGENT, HOST, AUTHORIZATION};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AcademicPeriod {
	academic_period: String,

	#[serde(skip_deserializing)]
	academic_period_description: String,
	#[serde(skip_deserializing)]
	academic_period_id: Option<String>,
	#[serde(skip_deserializing)]
	academic_period_status: bool,
	#[serde(skip_deserializing)]
	is_running_period: bool,
	#[serde(skip_deserializing)]
	term_begin_date: String,
	#[serde(skip_deserializing)]
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

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RoleCategories {
	#[serde(skip_deserializing)]
	name: String,
	roles: Vec<RoleCategory>
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UserProfile {
	#[serde(skip_deserializing)]
	user_id: String,
	#[serde(skip_deserializing)]
	full_name: String,
	#[serde(skip_deserializing)]
	person_code: String,
	#[serde(skip_deserializing)]
	email: String,
	#[serde(skip_deserializing)]
	user_picture_url: String,
	#[serde(skip_deserializing)]
	xP_point: f32,
	#[serde(skip_deserializing)]
	category_list: Vec<String>,

	role_categories: Vec<RoleCategories>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CustomParam {
	pub class_id: String,
	pub class_session_id: String,
	pub session_number: u32,

	#[serde(skip_deserializing)]
	pub class_session_content_id: Option<String>
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ScheduleDetails {
	pub date_start: String,
	pub date_end: String,
	pub title: String,
	pub content: String,

	#[serde(skip_deserializing)]
	pub location: Option<String>,

	#[serde(skip_deserializing)]
	pub location_value: Option<String>,

	#[serde(skip_deserializing)]
	pub schedule_type: String,
	pub custom_param: CustomParam,
	pub class_delivery_mode: String,

	#[serde(skip_deserializing)]
	pub delivery_mode: String,

	#[serde(skip_deserializing)]
	pub delivery_mode_desc: String,

	#[serde(skip_deserializing)]
	pub academic_career_desc: String,

	#[serde(skip_deserializing)]
	pub institution_desc: String,

	#[serde(skip_deserializing)]
	pub organization_role_id: String
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Schedule {
	#[serde(rename(deserialize = "Schedule"))]
	pub schedule: Vec<ScheduleDetails>,
	
	#[serde(skip_deserializing)]
	pub date_start: String
}

impl fmt::Display for Schedule {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		for class in &self.schedule {
			write!(f, "> Class Title: **{}**\n> Subject: **{}**\n> Start : **{}**\n> End: **{}**\n> Session: **{}**\n> Class Delivery Mode: **{}**\n> [Session link](https://newbinusmaya.binus.ac.id/lms/course/{}/session/{})\n\n", 
				class.title, class.content.clone(), 
				NaiveDateTime::parse_from_str(class.date_start.as_str(), "%FT%X").unwrap(), 
				NaiveDateTime::parse_from_str(class.date_end.as_str(), "%FT%X").unwrap(), 
				class.custom_param.session_number, 
				class.class_delivery_mode,
				class.custom_param.class_id,
				class.custom_param.class_session_id
			)?;
		}

		Ok(())
	}
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ClassSessionProgress {
	#[serde(skip_deserializing)]
	completed: u8,
	#[serde(skip_deserializing)]
	in_progress: u8,
	#[serde(skip_deserializing)]
	not_started: u8
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Lecturer {
	#[serde(skip_deserializing)]
	id: String,
	#[serde(skip_deserializing)]
	name: String,
	#[serde(skip_deserializing)]
	picture_url: String,
	#[serde(skip_deserializing)]
	role: String,
	#[serde(skip_deserializing)]
	user_code: String
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Resource {
	#[serde(skip_deserializing)]
	pub android_redirect_url: Option<String>,

	#[serde(skip_deserializing)]
	pub assessment_type: Option<String>,

	#[serde(skip_deserializing)]
	pub due_date: String,
	pub duration: Option<String>,
	pub id: String,

	#[serde(skip_deserializing)]
	pub index: String,

	#[serde(skip_deserializing)]
	pub ios_redirect_url: Option<String>,

	#[serde(skip_deserializing)]
	pub is_open: bool,

	#[serde(skip_deserializing)]
	pub is_overdue: bool,

	#[serde(skip_deserializing)]
	pub last_updated_date: String,
	pub name: String,
	pub progress_stamp: u8,
	pub progress_status: u8,

	#[serde(skip_deserializing)]
	pub resource_last_updated_date: String,

	#[serde(skip_deserializing)]
	pub resource_status: String,
	pub resource_type: String,

	#[serde(skip_deserializing)]
	pub thumbnail: Option<String>,

	#[serde(skip_deserializing)]
	pub times_accessed: u8,

	#[serde(skip_deserializing)]
	pub token: Option<String>,

	#[serde(rename(deserialize = "type"))]
	pub material_type: Option<String>,
	
	#[serde(skip_deserializing)]
	pub url: Option<String>
}

#[derive(Deserialize, Debug)]
#[serde(transparent)]
pub struct ResourceList {
	pub resources: Vec<Resource>
}

impl fmt::Display for ResourceList {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		for resource in &self.resources.clone() {
			let duration = {
				if let Some(i) = resource.duration.clone() {
					(i.parse::<u32>().unwrap() / 60).to_string()
				} else {
					"?".to_string()
				}
			};

			let progess_status = {
				if resource.progress_status == 2 || resource.progress_stamp == 1 {
					"Completed"
				} else if resource.progress_status == 1 {
					"In progress"
				} else {
					"Not started"
				}
			};

			write!(f, "> Name: **{}**\n> Duration: **{} min**\n> Type: **{}**\n> Status: **{}**\n\n", 
				resource.name, 
				duration,
				resource.resource_type,
				progess_status
			)?;
		}

		Ok(())
	}
}

#[derive(Debug, Deserialize)]
#[serde(transparent)]
pub struct SubTopics {
	subtopics: Vec<String> 
}

impl fmt::Display for SubTopics {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		for topic in &self.subtopics {
			write!(f, "- {}\n", topic)?;
		}
        Ok(())
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SessionDetails {
	#[serde(skip_deserializing)]
	pub class_delivery_mode: String,
	pub class_session_progress: ClassSessionProgress,
	pub course_sub_topic: SubTopics,
	pub date_end: String,
	pub date_start: String,
	pub delivery_mode: String,

	#[serde(skip_deserializing)]
	pub delivery_mode_desc: String,
	pub end_date_session_utc: String,

	#[serde(skip_deserializing)]
	pub is_ended: bool,
	pub join_url: Option<String>,

	#[serde(skip_deserializing)]
	pub lecturers: Vec<Lecturer>,

	#[serde(skip_deserializing)]
	pub meeting_end: String,

	#[serde(skip_deserializing)]
	pub meeting_start: String,
	pub resources: ResourceList,
	pub session_number: u8,
	pub start_date_session_utc: String,

	#[serde(skip_deserializing)]
	pub status: Option<String>,
	pub topic: String,

	#[serde(skip_deserializing)]
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
	#[serde(skip_deserializing)]
	pub crse_id: String,
	pub institution: String,

	pub lecturers: Vec<Lecturer>,
	#[serde(skip_deserializing)]

	pub n_sksp: String,
	#[serde(skip_deserializing)]

	pub n_skst: String,
	#[serde(skip_deserializing)]
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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimpleLecturer {
	#[serde(skip_deserializing)]
	id: String,
	#[serde(skip_deserializing)]
	name: String,
	#[serde(skip_deserializing)]
	picture_url: String
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimpleResource {
	duration: Option<String>,
	jumlah: Option<String>,
	r#type: Option<String>
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OngoingClass {
	#[serde(skip_deserializing)]
	academic_career_desc: String,

	#[serde(skip_deserializing)]
	class_campus_name: String,
	#[serde(skip)]
	class_code: String,

	#[serde(skip_deserializing)]
	class_delivery_mode: String,
	class_id: String,

	#[serde(skip_deserializing)]
	class_room_number: Option<String>,

	#[serde(skip_deserializing)]
	course_code: String,
	course_component: String,

	#[serde(skip_deserializing)]
	course_id: String,
	course_name: String,
	date_end: String,

	#[serde(skip_deserializing)]
	date_start: String,
	delivery_mode: String,

	#[serde(skip_deserializing)]
	delivery_mode_desc: String,
	id: String,

	#[serde(skip_deserializing)]
	institution_desc: String,

	#[serde(skip_deserializing)]
	is_ended: bool,

	#[serde(skip_deserializing)]
	lecturers: Vec<SimpleLecturer>,
	meeting_start: String,

	#[serde(skip_deserializing)]
	resource_id: Option<String>,
	resources: Vec<Option<SimpleResource>>,
	session_id: String,
	session_number: u8,

	#[serde(skip_deserializing)]
	session_progress: u8,

	#[serde(skip_deserializing)]
	url: Option<String>
}

#[derive(Debug, Deserialize)]
#[serde(transparent)]
pub struct OngoingClasses {
	pub ongoing_classes: Vec<OngoingClass>
}

impl fmt::Display for OngoingClasses {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		for ongoing_class in &self.ongoing_classes{
			let progress_status = {
				if ongoing_class.resources.is_empty() {
					"Completed"
				} else {
					"Incomplete"
				}
			};

			let now = chrono::offset::Local::now();
			let end_date = Local.from_local_datetime(
				&NaiveDateTime::parse_from_str(ongoing_class.date_end.as_str(), "%FT%X").unwrap()).unwrap();
			let time_left = end_date - now;
			write!(f, "> Class Component: **{}**\n> Course Name: **{}**\n> Time Left: **{} min**\n> Session: **{}**\n> Delivery Mode: **{}**\n> Status: **{}**\n> [Session Link](https://newbinusmaya.binus.ac.id/lms/course/{}/session/{})\n\n",
				ongoing_class.course_component, 
				ongoing_class.course_name, 
				time_left.num_minutes(),
				ongoing_class.session_number,
				ongoing_class.delivery_mode,
				progress_status,
				ongoing_class.class_id,
				ongoing_class.session_id
			)?;
		}
		Ok(())
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpcomingClass {
	#[serde(skip)]
	academic_career_desc: String,

	#[serde(skip)]
	class_campus_name: Option<String>,
	#[serde(skip)]
	class_code: String,

	#[serde(skip)]
	class_delivery_mode: String,
	class_id: String,

	#[serde(skip)]
	class_room_number: Option<String>,

	#[serde(skip)]
	course_code: String,
	course_component: String,

	#[serde(skip)]
	course_id: String,
	course_name: String,
	date_end: String,
	date_start: String,
	delivery_mode: String,

	#[serde(skip)]
	delivery_mode_desc: String,

	#[serde(skip)]
	id: String,

	#[serde(skip)]
	institution_desc: String,

	#[serde(skip)]
	is_ended: bool,

	#[serde(skip)]
	is_has_ongoing_class: bool,
	join_url: Option<String>,

	#[serde(skip)]
	lecturers: Vec<SimpleLecturer>,

	#[serde(skip)]
	meeting_start: String,

	#[serde(skip)]
	resource_id: String,

	#[serde(skip)]
	resources: Vec<Option<SimpleResource>>,
	session_id: String,
	session_number: u8,

	#[serde(skip)]
	session_progress: u8
}

impl fmt::Display for UpcomingClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let start_datetime = NaiveDateTime::parse_from_str(self.date_start.as_str(), "%FT%X").unwrap();
        write!(f, "**Class Zoom Link**\n{}\n\n**Session Info**\n> Class Component: **{}**\n> Course Name: **{}**\n> Time Start: **{}**\n> Session: **{}**\n> Delivery Mode: **{}**\n> [Session link](https://newbinusmaya.binus.ac.id/lms/course/{}/session/{})\n",
			self.join_url.clone().unwrap_or("No link".to_string()), 
			self.course_component, 
			self.course_name, 
			start_datetime,
			self.session_number, 
			self.delivery_mode,
			self.class_id,
			self.session_id
		)?;
		Ok(())
    }

}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OngoingClassResponse {
	pub data: OngoingClasses,
	pub is_has_upcoming_class: bool
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

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Announcement {
	academic_career_desc: String,
	announcement_master_id: String,
	pub id: String,
	title: String,
	start_date: String,
	end_date: String,
	#[serde(skip)]
	is_read: bool,
	#[serde(skip)]
	is_mandatory: bool,
	#[serde(skip)]
	link_url: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all ="camelCase")]
pub struct AnnouncementResponse {
	pub announcements: Vec<Announcement>,
	pub max_page: u8,
	pub page_number: u8,
	pub total_data: u8
}

impl fmt::Display for AnnouncementResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, announcement) in self.announcements.clone().into_iter().enumerate() {
			write!(f, "{}. **{}**\n", i + 1, announcement.title)?;
		}

		Ok(())
    }
}

#[derive(Deserialize, Debug)]
#[serde(transparent)]
pub struct Links {
	links: Vec<Option<String>>
}

impl fmt::Display for Links {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		for link in &self.links {
			if let Some(link) = link {
				write!(f, "{}\n", link)?;
			} else {
				write!(f, "No link")?;
			}
		}

		Ok(())
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all ="camelCase")]
pub struct AnnouncementDetails {
	academic_career_desc: String,
	pub attachment_links: Links,
	pub content: String,
	end_date: String,
	institution_desc: Option<String>,
	is_mandatory: bool,
	link_url: Option<String>,
	start_date: String,
	pub title: String
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
		headers.insert(HeaderName::from_static("institution"), HeaderValue::from_static("BNS01"));
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

	pub async fn get_schedule(&self, date: &NaiveDate) -> Result<Option<Schedule>, reqwest::Error> {
		let user_profile: UserProfile = BinusmayaAPI::get_user_profile(self).await.expect("Error in getting user profile");
		let mut headers = HeaderMap::new();
		headers.extend(BinusmayaAPI::init_full_header(self, &user_profile).await);
		headers.insert(HOST, HeaderValue::from_static("func-bm7-schedule-prod.azurewebsites.net"));

		let role_activity = RoleActivity::new(user_profile.role_categories[0].roles[0].clone());
		let mut role_activities = Vec::new();
		role_activities.push(role_activity);


		let client = reqwest::Client::new();
		let response = client
			.post(format!("https://func-bm7-schedule-prod.azurewebsites.net/api/Schedule/Date-v1/{}", date.to_string()))
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

		let client = reqwest::Client::builder()
			.connection_verbose(true)
			.build()?;
		let res = client
			.get("https://apim-bm7-prod.azure-api.net/func-bm7-course-prod/Class/Active/Student")
			.headers(headers)
			.send().await?;

		if res.status() != reqwest::StatusCode::OK {
			panic!("status: {}\n{}", res.status(), res.text().await?);
		}
		let classes = res.json::<ClassVec>().await.expect("Something's wrong when parsing");

		Ok(classes)
	}

	pub async fn get_class_details(&self, class_id: String) -> Result<ClassDetails, reqwest::Error> {
		let user_profile: UserProfile = BinusmayaAPI::get_user_profile(self).await.expect("Error in getting user profile");

		let mut headers = HeaderMap::new();
		headers.extend(BinusmayaAPI::init_full_header(self, &user_profile).await);

		let client = reqwest::Client::builder()
			.connection_verbose(true)
			.build()?;
		let response = client
			.get(format!("https://apim-bm7-prod.azure-api.net/func-bm7-course-prod/ClassSession/Class/{}/Student", class_id))
			.headers(headers)
			.send().await.expect("something's wrong")
			.json::<ClassDetails>().await.expect("Something's wrong when parsing response");

		Ok(response)
	}

	pub async fn update_student_progress(&self, resource_id: &String) -> Result<reqwest::StatusCode, reqwest::Error> {
		let user_profile: UserProfile = BinusmayaAPI::get_user_profile(self).await.expect("Error in getting user profile");

		let mut headers = HeaderMap::new();
		headers.extend(BinusmayaAPI::init_full_header(self, &user_profile).await);

		let client = reqwest::Client::new();
		let response = client
			.post("https://apim-bm7-prod.azure-api.net/func-bm7-course-prod/StudentProgress")
			.headers(headers)
			.json::<StudentProgressPayload>(&StudentProgressPayload{resource_id: resource_id.to_string(), status: 2})
			.send().await.expect("Something's wrong when sending request");

		Ok(response.status())
	}

	pub async fn get_ongoing_sessions(&self) -> Result<OngoingClassResponse, reqwest::Error> {
		let user_profile: UserProfile = BinusmayaAPI::get_user_profile(self).await.expect("Error in getting user profile");

		let mut headers = HeaderMap::new();
		headers.extend(BinusmayaAPI::init_full_header(self, &user_profile).await);

		let client = reqwest::Client::new();

		let res = client
			.get("https://apim-bm7-prod.azure-api.net/func-bm7-course-prod/ClassSession/Ongoing/student")
			.headers(headers)
			.send().await.expect("something's wrong when sending request")
			.json::<OngoingClassResponse>().await.expect("Something's wrong when parsing response");

		Ok(res)
	}

	pub async fn get_upcoming_sessions(&self) -> Result<Option<UpcomingClass>, reqwest::Error> {
		let user_profile: UserProfile = BinusmayaAPI::get_user_profile(self).await.expect("Error in getting user profile");

		let mut headers = HeaderMap::new();
		headers.extend(BinusmayaAPI::init_full_header(self, &user_profile).await);

		let client = reqwest::Client::new();
		let res = client
			.get("https://apim-bm7-prod.azure-api.net/func-bm7-course-prod/ClassSession/Upcoming/student")
			.headers(headers)
			.send().await.expect("something's wrong when sending request")
			.json::<Option<UpcomingClass>>().await.unwrap_or(None );

		Ok(res)
	}

	pub async fn get_announcement(&self, page_number: u8) -> Result<AnnouncementResponse, reqwest::Error> {
		let user_profile: UserProfile = self.get_user_profile().await?;

		let mut headers = HeaderMap::new();
		headers.extend(self.init_full_header(&user_profile).await);

		let client = reqwest::Client::new();
		let res = client
			.get(format!("https://apim-bm7-prod.azure-api.net/func-bm7-notification-prod/Announcements/PageSize/100/PageNumber/{}", page_number))
			.headers(headers)
			.send().await.expect("Something's wrogn when sending request")
			.json::<AnnouncementResponse>().await.expect("Something's wrong when parsing response");
		
		Ok(res)
	}

	pub async fn get_announcement_details(&self, id: &String) -> Result<Option<AnnouncementDetails>, reqwest::Error> {
		let user_profile: UserProfile = self.get_user_profile().await?;

		let mut headers = HeaderMap::new();
		headers.extend(self.init_full_header(&user_profile).await);

		let client = reqwest::Client::new();
		let res = client
			.get(format!("https://apim-bm7-prod.azure-api.net/func-bm7-notification-prod/Announcements/NoRead/{}", id))
			.headers(headers)
			.send().await?
			.json::<AnnouncementDetails>().await?;
		
		Ok(Some(res))
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::env;

	#[tokio::test]
	async fn get_announcement_test() {
		let token = env::var("BEARER_TOKEN").unwrap();
		let binusmaya_api = BinusmayaAPI{token};
		let res = binusmaya_api.get_announcement(1).await.unwrap();
		println!("{:#?}", res);
		assert_eq!(res.page_number, 1);
	}

	#[tokio::test]
	async fn get_announcement_details() {
		let token = env::var("BEARER_TOKEN").unwrap();
		let binusmaya_api = BinusmayaAPI{token};
		let res = binusmaya_api.get_announcement_details(&String::from("0167b34e-bdfc-4a41-8a94-bf08061366f4")).await.unwrap();
		println!("{:#?}", res);
		assert_eq!(res.unwrap().title.is_empty(), false);
	}
}