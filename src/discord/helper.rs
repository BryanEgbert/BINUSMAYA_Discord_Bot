use std::{error::Error, fmt::Display, str::FromStr, fs::read_to_string};

use csv_async::AsyncReaderBuilder;
use futures::StreamExt;
use serenity::{
    builder::{CreateActionRow, CreateButton, CreateSelectMenu, CreateSelectMenuOption},
    model::interactions::message_component::ButtonStyle,
};

use crate::{consts::{OLDBINUSMAYA_USER_FILE, OLDBINUSMAYA_USER_DATA}, api::old_binusmaya_api::OldBinusmayaAPI};

use super::discord::OldBinusmayaUserRecord;

#[derive(PartialEq)]
pub enum Nav {
    Previous,
    Next,
}

impl Display for Nav {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Previous => write!(f, "<"),
            Self::Next => write!(f, ">"),
        }
    }
}

#[derive(Debug)]
pub struct ParseError(pub String);

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Failed to parse {}", self.0)
    }
}

impl Error for ParseError {}

impl FromStr for Nav {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "<" => Ok(Nav::Previous),
            ">" => Ok(Nav::Next),
            _ => Err(ParseError(s.to_string())),
        }
    }
}

impl Nav {
    fn button(&self) -> CreateButton {
        let mut btn = CreateButton::default();
        btn.custom_id(self.to_string().to_ascii_lowercase());
        btn.label(self);
        btn.style(ButtonStyle::Primary);

        btn
    }

    pub fn action_row() -> CreateActionRow {
        let mut ar = CreateActionRow::default();
        ar.add_button(Nav::Previous.button());
        ar.add_button(Nav::Next.button());

        ar
    }
}

pub async fn update_cookie(user_id: Option<u64>) {
    let oldbinusmaya_content = read_to_string(OLDBINUSMAYA_USER_FILE).expect("Something's wrong when reading a file");

    let rdr = AsyncReaderBuilder::new()
        .has_headers(false)
        .create_deserializer(oldbinusmaya_content.as_bytes());

    let mut records = rdr.into_deserialize::<OldBinusmayaUserRecord>();
    if let Some(id) = user_id {
         while let Some(record) = records.next().await {
            let record = record.unwrap();
            if record.member_id == id {
                let old_binusmaya_api = OldBinusmayaAPI::login(&record.binusian_data, &record.user_credential).await;
                OLDBINUSMAYA_USER_DATA.lock().await.insert(record.member_id, old_binusmaya_api.cookie);
                break;
            }
        }
    } else {
        while let Some(record) = records.next().await {
            let record = record.unwrap();
            let old_binusmaya_api = OldBinusmayaAPI::login(&record.binusian_data, &record.user_credential).await;
            OLDBINUSMAYA_USER_DATA.lock().await.insert(record.member_id, old_binusmaya_api.cookie);
        }
    }
}

pub async fn select_menu(menu_options: Vec<CreateSelectMenuOption>) -> CreateSelectMenu {
    let mut menu = CreateSelectMenu::default();
    menu.custom_id("academic_period_select");
    menu.placeholder("No academic period selected");
    menu.options(|f| {
        for option in menu_options {
            f.add_option(option);
        }

        f
    });

    menu
}
