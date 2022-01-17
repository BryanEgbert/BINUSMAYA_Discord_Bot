#![allow(non_snake_case)]
#![allow(dead_code)]
pub mod third_party;
pub mod discord;
pub mod binusmaya;
pub mod consts;
mod dropbox_api;

#[macro_use]
extern crate lazy_static;

use tokio::fs::{File, write};
use std::process::Command;
use discord::discord::run;

use consts::{LOGIN_FILE, USER_FILE};

#[tokio::main]
async fn main() {
    fetch_file().await;
    start_third_party_apps();
    run().await;
}

fn start_third_party_apps() {
    Command::new("./chromedriver")
        .arg("--port=4444")
        .spawn()
        .expect("Failed to run chrome driver");
    
    Command::new("sh")
        .args(["./browsermob-proxy-2.1.4/bin/browsermob-proxy", "--address", "localhost", "--port", "8082"])
        .spawn()
        .expect("Failed to start browsermob-proxy");
}

async fn fetch_file() {
    File::create(LOGIN_FILE).await.expect("Error in creating login.txt");

    File::create(USER_FILE).await.expect("Error in creating ");

    let user_content = dropbox_api::download_file(USER_FILE.to_string()).await.unwrap();

    if let Some(content) = user_content {
        write(USER_FILE, content.as_bytes()).await.unwrap();
    }

    println!("File created successfully");
}