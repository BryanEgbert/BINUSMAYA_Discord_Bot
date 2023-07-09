use chrono::{DateTime, Local};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UserCredential {
    pub email: String,
    pub password: String
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

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UserBinusianData {
    pub binusian_id: String,
    pub display_name: String,
    pub user_id: String,
    pub role_id: u8,
    pub specific_role_id: u8,
}

impl UserBinusianData {
    pub fn init(binusian_data: &BinusianData) -> Self {
        let user_binusian_data = UserBinusianData {
            binusian_id: binusian_data.binusian_id.clone(),
            display_name: format!("{} {}", binusian_data.first_name, binusian_data.last_name),
            user_id: binusian_data.nim.clone(),
            role_id: 2,
            specific_role_id: 104,
        };

        user_binusian_data
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OldBinusmayaUser {
    pub member_id: u64,
    pub user_credential: UserCredential,
    pub binusian_data: UserBinusianData,
    pub cookie: String ,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NewBinusmayaUser {
    pub member_id: u64,
    pub auth: String,
    pub last_registered: DateTime<Local>,
}