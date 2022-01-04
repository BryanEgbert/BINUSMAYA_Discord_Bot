#![allow(non_snake_case)]
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
    let chrome_binary = env::args().nth(1);

    if let Some(path) = chrome_binary {
		if Path::new(&path).exists() {
            if env::consts::OS == "linux" {
                Command::new("./chromedriver")
                    .arg("--port=4444")
                    .spawn()
                    .expect("Failed to run chrome driver");
            
                Command::new("sh")
                    .args(["./browsermob-proxy-2.1.4/bin/browsermob-proxy", "--address", "localhost", "--port", "8082"])
                    .spawn()
                    .expect("Failed to start browsermob-proxy");
            } else if env::consts::OS == "windows" {
                Command::new(".\\chromedriver.exe")
                    .arg("--port=4444")
                    .spawn()
                    .expect("Failed to run chrome driver, please download chrome driver version 96 here: https://chromedriver.storage.googleapis.com/index.html?path=96.0.4664.45/\ndon't forget to install google chrome version 96 too");
            
                Command::new(".\\browsermob-proxy-2.1.4\\bin\\browsermob-proxy.bat")
                    .args(["--address", "localhost", "--port", "8082"])
                    .spawn()
                    .expect("Failed to start browsermob-proxy, please install it here: http://bmp.lightbody.net");
            } else {
                println!("Your OS is not supported");
                abort();
            }
		} else {
			panic!("Chrome binary path doesn't exists");
		}
	} else {
		panic!("Please enter your chrome binary path");
	}

    
    run().await;
}