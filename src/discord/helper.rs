use std::{error::Error, fmt::Display, str::FromStr};

use serenity::{
    builder::{CreateActionRow, CreateButton},
    model::interactions::message_component::ButtonStyle,
};

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
