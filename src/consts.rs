use std::{collections::HashMap, env, sync::Arc};
use tokio::sync::Mutex;
use serenity::utils::Colour;
use crate::discord::discord::UserAuthInfo;

pub const PRIMARY_COLOR: Colour = Colour::BLUE;
pub const USER_FILE: &str = "user_data.csv";
pub const LOGIN_FILE: &str = "last_login.txt";
lazy_static!{
	pub static ref USER_DATA: Arc<Mutex<HashMap<u64, UserAuthInfo>>> = Arc::new(Mutex::new(HashMap::new()));
	pub static ref CHROME_BINARY: Mutex<String> = Mutex::new(env::var("GOOGLE_CHROME_SHIM").unwrap());
}