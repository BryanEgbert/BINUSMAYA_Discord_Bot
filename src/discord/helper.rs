use std::{error::Error, fmt::Display, str::FromStr};



use serenity::{
    builder::{CreateActionRow, CreateButton, CreateSelectMenu, CreateSelectMenuOption},
    model::application::component::ButtonStyle,
};

use crate::{consts, api::old_binusmaya_api::OldBinusmayaAPI, repository};

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

pub async fn update_cookie_all() {
    let user_data = consts::OLD_BINUSMAYA_REPO.get_all().unwrap();

    for user in user_data {
        let old_binusmaya_api = OldBinusmayaAPI::login(&user.binusian_data, &user.user_credential).await;

        consts::OLD_BINUSMAYA_REPO.update_cookie_by_id(&user.member_id, old_binusmaya_api.cookie).unwrap();
    }
}

pub async fn update_cookie(repository: &repository::old_binusmaya_repository::OldBinusmayaRepository, member_id: &u64) {
    let user = repository.get_by_id(member_id).unwrap().unwrap();

    let old_binusmaya_api = OldBinusmayaAPI::login(&user.binusian_data, &user.user_credential).await;
    repository.update_cookie_by_id(member_id, old_binusmaya_api.cookie).unwrap();
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
