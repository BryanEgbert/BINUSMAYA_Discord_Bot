#![allow(non_snake_case)]
#![allow(dead_code)]
pub mod api;
pub mod consts;
pub mod discord;
pub mod third_party;

#[macro_use] extern crate lazy_static;
#[macro_use] extern crate magic_crypt;

use discord::discord::run;
use std::process::Command;
use tokio::fs::{write, File};

use consts::{LOGIN_FILE, NEWBINUSMAYA_USER_FILE};

use crate::consts::OLDBINUSMAYA_USER_FILE;

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
        .args([
            "./browsermob-proxy-2.1.4/bin/browsermob-proxy",
            "--address",
            "localhost",
            "--port",
            "8082",
        ])
        .spawn()
        .expect("Failed to start browsermob-proxy");
}

async fn fetch_file() {
    File::create(LOGIN_FILE)
        .await
        .expect("Error in creating login.txt");

    File::create(NEWBINUSMAYA_USER_FILE).await.expect("Error in creating new binusmaya file");

    File::create(OLDBINUSMAYA_USER_FILE).await.expect("Error in creating old binusmaya file");

    let user_content = api::dropbox_api::download_file(NEWBINUSMAYA_USER_FILE.to_string())
        .await
        .unwrap();

    if let Some(content) = user_content {
        write(NEWBINUSMAYA_USER_FILE, content.as_bytes()).await.unwrap();
    }

    println!("File created successfully");
}
