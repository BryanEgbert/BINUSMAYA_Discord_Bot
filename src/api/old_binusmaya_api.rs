use std::{fmt::Display, collections::HashMap, io::Cursor, fs::File};

use reqwest::{header::{HeaderMap, CONTENT_TYPE, HOST, HeaderValue, ORIGIN, REFERER, COOKIE, HeaderName, SET_COOKIE}, redirect::Policy};
use serde::{Deserialize, Deserializer, Serialize};
use thirtyfour::Cookie;

use crate::discord::discord::{UserBinusianData, UserCredential};


#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct SATPoint {
	pub activity_type: String,
	pub points: u8,
	pub target_points: u8,
	pub total_points: u8
}

#[derive(Deserialize, Debug)]
#[serde(transparent)]
pub struct SATPoints {
	pub sat_points: Vec<SATPoint>
}

impl Display for SATPoints {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for sat_point in &self.sat_points {
			write!(f, "{} - **{}** point(s)\n", sat_point.activity_type, sat_point.points)?;
		}

		Ok(())
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct ComServ {
	community_service_type: String,
	points: u8,
	total_points: u8,
	target_points: u8
}

#[derive(Deserialize, Debug)]
#[serde(transparent)]
pub struct ComServList {
	list: Vec<ComServ>
}

impl Display for ComServList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for comserv in &self.list {
			write!(f, "{} - **{}** hour(s)", comserv.community_service_type, comserv.points)?;
		}

		Ok(())
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Course {
	#[serde(rename = "CRSE_ID")] pub crse_id: String,
	#[serde(rename = "COURSENAME")] pub course_name: String,
	#[serde(rename = "COURSEID")] pub course_id: String,
	#[serde(rename = "CLASS_SECTION")] pub class_section: String,
	#[serde(rename = "STRM")] pub strm: String,
	#[serde(rename = "SSR_COMPONENT")] pub ssr_component: String,
	#[serde(rename = "CLASS_NBR")] pub class_nbr: String,
	#[serde(rename = "VBID")] pub vbid: Option<String>,
	pub redeem_status: Option<u8>,
	pub status_online_learning: String,
	#[serde(rename = "StatusGLS")] pub status_gls: String,
	pub status_blended: String
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Courses {
	pub courses: Vec<Course>
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Assignment {
	#[serde(rename = "AssignmentFrom")] pub assignment_from: String,
	#[serde(rename = "Title")] pub title: String,
	#[serde(rename = "Date")] pub date: String,
	pub deadline_duration: String,
	pub deadline_time: String,
	pub assignment_path_location: String,
	#[serde(rename = "assignmentURL")] pub assignment_url: Option<String>,
	#[serde(rename = "StudentAssignmentID")] pub student_assignment_id: u32,
	#[serde(rename = "courseOutlineTopicID")] pub course_outline_topic_id: u32,
	#[serde(rename = "courseAssignmentID")] pub course_assignment_id: Option<u16>,
	#[serde(rename = "webcontent")] pub web_content: String
}

#[derive(Deserialize, Debug, Clone)]
#[serde(transparent)]
pub struct AssignmentList {
	pub assignments: Vec<Assignment>
}

impl Display for AssignmentList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		if self.assignments.is_empty() {
			write!(f, "No assignment")?;
		}
		for assignment in &self.assignments {
			write!(f, "> Title: **{}**\n> Due datetime: **{} {}**\n\n",
				assignment.title, assignment.deadline_duration, assignment.deadline_time)?;
		}

		Ok(())
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct BinusianData {
	#[serde(rename = "ACAD_CAREER")] pub acad_career: String,
	#[serde(rename = "BinusianID")] pub binusian_id: String,
	#[serde(rename = "CAMPUS")] #[serde(skip)] pub campus: String,
	#[serde(rename = "EMAIL_ADDR")] pub email: String,
	#[serde(rename = "FIRST_NAME")] pub first_name: String,
	#[serde(rename = "INSTITUTION")] pub institution: String,
	#[serde(rename = "LAST_NAME")] pub last_name: String,
	#[serde(rename = "NIM")] pub nim: String,
	#[serde(rename = "Queue_history")] #[serde(skip_deserializing)] pub queue_history: u8,
	#[serde(skip_deserializing)] pub feedback: u8,
	#[serde(skip_deserializing)] pub live_chat: u8,
	#[serde(skip_deserializing)] pub phone: String
}

fn str_or_u64<'de, D>(deserializer: D) -> Result<String, D::Error>
	where D: Deserializer<'de> 
{
	#[derive(Deserialize)]
	#[serde(untagged)]
	enum StrOrU64 {
		Str(String),
		U64(u64),
	}

	Ok(match StrOrU64::deserialize(deserializer)? {
		StrOrU64::Str(v) => v,
		StrOrU64::U64(v) => v.to_string(),
	})
}

#[derive(Deserialize, Debug)]
pub struct SessionStatus {
	#[serde(rename = "RoleID")] #[serde(deserialize_with = "str_or_u64")] pub role_id: String,
	#[serde(rename = "SessionStatus")] pub session_status: u8
}

#[derive(Debug)]
pub struct OldBinusmayaAPI {
	pub cookie: String
}

impl OldBinusmayaAPI {
	async fn init_client(&self) -> reqwest::Client {
		let mut headers = HeaderMap::new();
		headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json;charset=utf-8"));
		headers.insert(HOST, HeaderValue::from_static("binusmaya.binus.ac.id"));
		headers.insert(ORIGIN, HeaderValue::from_static("https://binusmaya.binus.ac.id"));
		headers.insert(REFERER, HeaderValue::from_static("https://binusmaya.binus.ac.id/newStudent/"));
		headers.insert(COOKIE, HeaderValue::from_str(self.cookie.as_str()).unwrap());
		headers.insert(HeaderName::from_static("content-length"), HeaderValue::from_static("0"));

		let client = reqwest::Client::builder()
			.cookie_store(true)
			.default_headers(headers)
			.build().unwrap();

		client
	}

	pub async fn get_sat(&self) -> Result<SATPoints, reqwest::Error> {
		let client = self.init_client().await;
		let response = client
			.post("https://binusmaya.binus.ac.id/services/ci/index.php/sat/studentactivitytranscript/GetStudentActivityPoint")
			.send()
			.await?;

		Ok(response
			.json::<SATPoints>()
			.await?)
	}

	pub async fn get_comnunity_service(&self) -> Result<ComServList, reqwest::Error> {
		let client = self.init_client().await;
		let response = client
			.post("https://binusmaya.binus.ac.id/services/ci/index.php/sat/studentactivitytranscript/GetCommunityServices")
			.send()
			.await?
			.json::<ComServList>()
			.await?;

		Ok(response)
	}

	pub async fn get_courses(&self) -> Result<Option<Courses>, reqwest::Error> {
		let client = self.init_client().await;
		let response = client
			.get("https://binusmaya.binus.ac.id/services/ci/index.php/student/init/getCourses")
			.send()
			.await?
			.json::<Option<Courses>>()
			.await?;

		Ok(response)
	}

	pub async fn get_course_menu_list(&self) -> Result<serde_json::Value, reqwest::Error> {
		let client = self.init_client().await;
		let response = client
			.post("https://binusmaya.binus.ac.id/services/ci/index.php/student/init/getStudentCourseMenuCourses")
			.send()
			.await?
			.json()
			.await?;
			
		Ok(response)
	}

	pub async fn get_individual_assignments(&self, course_id: &str, crse_id: &str, strm: &str, ssr_component: &str,  class_number: &str) -> Result<AssignmentList, reqwest::Error> {
		let client = self.init_client().await;
		let response = client
			.get(format!("https://binusmaya.binus.ac.id/services/ci/index.php/student/classes/assignmentType/{}/{}/{}/{}/{}/01", course_id, crse_id, strm, ssr_component, class_number))
			.send()
			.await?
			.json::<AssignmentList>()
			.await?;

		Ok(response)
	}

	pub async fn get_group_assignments(&self, course_id: &str, crse_id: &str, strm: &str, ssr_component: &str,  class_number: &str) -> Result<AssignmentList, reqwest::Error> {
		let client = self.init_client().await;
		let response = client
			.get(format!("https://binusmaya.binus.ac.id/services/ci/index.php/student/classes/assignmentType/{}/{}/{}/{}/{}/02", course_id, crse_id, strm, ssr_component, class_number))
			.send()
			.await?
			.json::<AssignmentList>()
			.await?;

		Ok(response)
	}

	pub async fn get_binusian_data(&self) -> Result<BinusianData, reqwest::Error> {
		let client = self.init_client().await;
		let res = client
			.post("https://binusmaya.binus.ac.id/services/ci/index.php/general/getBinusianData")
			.send()
			.await?
			.json::<BinusianData>()
			.await?;

		Ok(res)
	}

	pub async fn check_session(&self) -> Result<SessionStatus, reqwest::Error> {
		let client = self.init_client().await;
		let res = client
			.get("https://binusmaya.binus.ac.id/services/ci/index.php/staff/init/check_session")
			.send()
			.await?
			.json::<SessionStatus>()
			.await?;

		Ok(res)
	}

	pub fn init_cookie(cookie: &Cookie) -> Self {
		let binusmaya_api = OldBinusmayaAPI {
			cookie: format!("{}={}", cookie.name(), cookie.value())
		};

		binusmaya_api
	}

	pub async fn login(binusian_data: &UserBinusianData, user_credential: &UserCredential) -> Self {
		let mut params = HashMap::with_capacity(7);
		params.insert("displayName", binusian_data.display_name.clone());
		params.insert("userName", user_credential.email.clone());
		params.insert("employeeID", binusian_data.binusian_id.clone());
		params.insert("UserID",binusian_data.user_id.clone());
		params.insert("RoleID", binusian_data.role_id.to_string());
		params.insert("SpecificRoleID", binusian_data.specific_role_id.to_string());
		params.insert("forcelogin", "forcelogin".to_string());
		
		let client = reqwest::Client::builder()
			.redirect(Policy::none())
			.cookie_store(true)
			.build().unwrap();

		let res = client
			.post("https://binusmaya.binus.ac.id//LoginAD.php")
			.header(CONTENT_TYPE, HeaderValue::from_static("application/x-www-form-urlencoded"))
			.header(HOST, HeaderValue::from_static("binusmaya.binus.ac.id"))
			.form(&params)
			.send()
			.await.unwrap();

		let cookie = res.headers().get(SET_COOKIE).unwrap().to_str().unwrap().to_string();

		let binusmaya_api= OldBinusmayaAPI {
			cookie
		};
		
		binusmaya_api
	}

	pub async fn download_assignment(&self, link: &str) -> Result<(), reqwest::Error> {
		let file_name_start_index = link.rfind("\\").unwrap();
		let file_name = &link[file_name_start_index+1..];
		let mut file = File::create(file_name).unwrap();

		let mut assignment_url = String::from("https://binusmaya.binus.ac.id/services/ci/index.php/general/downloadDocument/");
		assignment_url.push_str(link.replace("\\", "...").replace(" ", "%20").as_str());

		let client = self.init_client().await;
		let res= client
			.get(assignment_url)
			.send()
			.await?;

		let mut content = Cursor::new(res.bytes().await?);

		std::io::copy(&mut content, &mut file).unwrap();

		Ok(())
	}
}

#[cfg(test)]
mod tests {
use crate::{discord::helper::update_cookie, consts::OLDBINUSMAYA_USER_DATA};

use super::*;
	const COOKIE_VAL: &str = "PHPSESSID=j4f4hfv6lq17obfin5nbg3j926";

	#[tokio::test]
	async fn check_session() {
		let binusmaya_api = OldBinusmayaAPI {
			cookie: COOKIE_VAL.to_string()
		};

		let res = binusmaya_api.check_session().await.unwrap();
		println!("{:#?}", res);
	}
	#[tokio::test]
	async fn get_binusian_data() {
		let binusmaya_api = OldBinusmayaAPI {
			cookie: COOKIE_VAL.to_string()
		};

		let res = binusmaya_api.get_binusian_data().await.unwrap();

		println!("{:#?}", res);
	}
	#[tokio::test]
	async fn get_sat() {
		let binusmaya_api = OldBinusmayaAPI {
			cookie: COOKIE_VAL.to_string()
		};

		let res = binusmaya_api.get_sat().await.unwrap();

		println!("{:#?}", res);

		assert_eq!(res.sat_points.is_empty(), false);
	}

	#[tokio::test]
	async fn get_community_service() {
		let binusmaya_api = OldBinusmayaAPI {
			cookie: COOKIE_VAL.to_string()
		};

		let res = binusmaya_api.get_comnunity_service().await.unwrap();

		println!("{:#?}", res);

		assert_eq!(res.list.is_empty(), false);
	}

	#[tokio::test]
	async fn get_courses() {
		let binusmaya_api = OldBinusmayaAPI {
			cookie: COOKIE_VAL.to_string()
		};

		let res = binusmaya_api.get_courses().await.unwrap();

		println!("{:#?}", res);
	}

	#[tokio::test]
	async fn get_individual_assignments() {
		let binusmaya_api = OldBinusmayaAPI {
			cookie: COOKIE_VAL.to_string()
		};

		let res = binusmaya_api.get_individual_assignments("CHAR6013001", "021583", "2110", "LEC", "21679").await.unwrap();

		println!("{}", res);
	}

	#[tokio::test]
	async fn get_course_menu_list() {
		let binusmaya_api = OldBinusmayaAPI {
			cookie: COOKIE_VAL.to_string()
		};

		let res = binusmaya_api.get_course_menu_list().await.unwrap();

		println!("{:#?}", res); 
	}

	#[tokio::test]
	async fn update_cookie_test() {
		let user_data = OLDBINUSMAYA_USER_DATA.clone();
		let mut user_data_content = user_data.lock().await;
		user_data_content.insert(123, COOKIE_VAL.to_string());
		let cookie = user_data_content.get(&123).unwrap();
		let mut binusmaya_api = OldBinusmayaAPI {
			cookie: cookie.to_string()
		};

		let session_status = binusmaya_api.check_session().await.unwrap().session_status;
		println!("{}", session_status);

		if session_status == 0 {
			println!("update cookie");
			binusmaya_api = update_cookie(&123, binusmaya_api).await;
			println!("{:?}", binusmaya_api);
			user_data_content.insert(123, binusmaya_api.cookie.clone());
			println!("{:#?}", user_data_content);
		}

		let sat = binusmaya_api.get_sat().await.unwrap();
		println!("{:#?}", sat);
	}

	#[tokio::test]
	async fn download_assignment_test() {
		let binusmaya_api = OldBinusmayaAPI {
			cookie: COOKIE_VAL.to_string()
		};

		let res = binusmaya_api.download_assignment(r"general_course_outline\course_outline\assignment\RS1\010612\2020100113534300000581_Assignment 1 (Minggu ke-4) (Individual).docx").await.unwrap();
	}
}