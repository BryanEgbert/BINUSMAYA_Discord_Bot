#![allow(non_snake_case)]
#![allow(dead_code)]
pub mod third_party;
pub mod discord;
pub mod binusmaya;
pub mod commands;
pub mod consts;

#[macro_use]
extern crate lazy_static;

use discord::*;
use std::{process::{Command, abort}, env, path::Path};



#[tokio::main]
async fn main() {

    Command::new("./chromedriver")
        .arg("--port=4444")
        .spawn()
        .expect("Failed to run chrome driver");

    Command::new("sh")
        .args(["./browsermob-proxy-2.1.4/bin/browsermob-proxy", "--address", "localhost", "--port", "8082"])
        .spawn()
        .expect("Failed to start browsermob-proxy");
    
    run().await;
}