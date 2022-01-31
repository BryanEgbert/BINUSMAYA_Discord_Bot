use crate::discord::discord::NewBinusmayaUserAuthInfo;
use magic_crypt::MagicCrypt256;
use serenity::utils::Colour;
use thirtyfour::Cookie;
use std::{collections::HashMap, env, sync::Arc};
use tokio::sync::Mutex;

pub const PRIMARY_COLOR: Colour = Colour::BLUE;

pub const NEWBINUSMAYA_USER_FILE: &str = "user_data.csv";
pub const OLDBINUSMAYA_USER_FILE: &str = "old_binusmaya_user_data.csv";
pub const LOGIN_FILE: &str = "last_login.txt";

pub const NEW_BINUSMAYA: &str = "https://newbinusmaya.binus.ac.id";
pub const OLD_BINUSMAYA: &str = "https://binusmaya.binus.ac.id";

pub const CHROME_SERVER_URL: &str = "http://localhost:4444";

lazy_static! {
    pub static ref NEWBINUSMAYA_USER_DATA: Arc<Mutex<HashMap<u64, NewBinusmayaUserAuthInfo>>> =
        Arc::new(Mutex::new(HashMap::new()));
    pub static ref OLDBINUSMAYA_USER_DATA: Arc<Mutex<HashMap<u64, Cookie>>> =
        Arc::new(Mutex::new(HashMap::new()));
    pub static ref CHROME_BINARY: Mutex<String> =
        Mutex::new(env::var("GOOGLE_CHROME_SHIM").unwrap());
    pub static ref MAGIC_CRYPT: MagicCrypt256 = new_magic_crypt!("key", 256);
}
