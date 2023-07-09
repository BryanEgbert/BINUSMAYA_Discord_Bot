use crate::{discord::discord::NewBinusmayaUserAuthInfo, repository::{old_binusmaya_repository::OldBinusmayaRepository, new_binusmaya_repository::NewBinusmayaRepository}};
use magic_crypt::MagicCrypt256;
use serenity::utils::Colour;
use std::{collections::HashMap, env, sync::Arc};
use tokio::sync::Mutex;
use rusqlite;

pub const PRIMARY_COLOR: Colour = Colour::BLUE;

pub const NEWBINUSMAYA_USER_FILE: &str = "user_data.csv";
pub const OLDBINUSMAYA_USER_FILE: &str = "old_binusmaya_user_data.csv";
pub const LOGIN_FILE: &str = "last_login.txt";

pub const NEW_BINUSMAYA: &str = "https://newbinusmaya.binus.ac.id";
pub const OLD_BINUSMAYA: &str = "https://binusmaya.binus.ac.id";

pub const CHROME_SERVER_URL: &str = "http://192.168.100.25:9222";
pub const DB_NAME: &str = "bimay_discord_bot.db";


lazy_static! {
    pub static ref NEWBINUSMAYA_USER_DATA: Arc<Mutex<HashMap<u64, NewBinusmayaUserAuthInfo>>> =
        Arc::new(Mutex::new(HashMap::new()));
    pub static ref OLDBINUSMAYA_USER_DATA: Arc<Mutex<HashMap<u64, String>>> =
        Arc::new(Mutex::new(HashMap::new()));
    pub static ref CHROME_BINARY: Mutex<String> =
        Mutex::new(env::var("GOOGLE_CHROME_SHIM").unwrap());
    pub static ref MAGIC_CRYPT: MagicCrypt256 = new_magic_crypt!(env::var("SECRET_KEY").expect("expected SECRET KEY in env"), 256);
    pub static ref OLD_BINUSMAYA_REPO: OldBinusmayaRepository = OldBinusmayaRepository::new(rusqlite::Connection::open(DB_NAME).unwrap());
    pub static ref NEW_BINUSMAYA_REPO: NewBinusmayaRepository = NewBinusmayaRepository::new(rusqlite::Connection::open(DB_NAME).unwrap());
}
