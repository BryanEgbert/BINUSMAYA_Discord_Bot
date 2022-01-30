use crate::discord::discord::{NewBinusmayaUserAuthInfo, OldBinusmayaUserAuthInfo};
use serenity::utils::Colour;
use std::{collections::HashMap, env, sync::Arc};
use tokio::sync::Mutex;

pub const PRIMARY_COLOR: Colour = Colour::BLUE;
pub const USER_FILE: &str = "user_data.csv";
pub const LOGIN_FILE: &str = "last_login.txt";
pub const NEW_BINUSMAYA: &str = "https://newbinusmaya.binus.ac.id";
pub const OLD_BINUSMAYA: &str = "https://binusmaya.binus.ac.id";
lazy_static! {
    pub static ref NEWBINUSMAYA_USER_DATA: Arc<Mutex<HashMap<u64, NewBinusmayaUserAuthInfo>>> =
        Arc::new(Mutex::new(HashMap::new()));
    pub static ref OLDBINUSMAYA_USER_DATA: Arc<Mutex<HashMap<u64, OldBinusmayaUserAuthInfo>>> =
        Arc::new(Mutex::new(HashMap::new()));
    pub static ref CHROME_BINARY: Mutex<String> =
        Mutex::new(env::var("GOOGLE_CHROME_SHIM").unwrap());
}
