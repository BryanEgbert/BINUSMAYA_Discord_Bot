use std::collections::HashMap;

use tokio::sync::Mutex;

use serenity::utils::Colour;

use crate::discord::UserAuthInfo;

pub const PRIMARY_COLOR: Colour = Colour::BLUE;
lazy_static!{
	pub static ref USER_DATA: Mutex<HashMap<u64, UserAuthInfo>> = Mutex::new(HashMap::new());
	pub static ref CHROME_BINARY: Mutex<Option<String>> = Mutex::new(None);
}