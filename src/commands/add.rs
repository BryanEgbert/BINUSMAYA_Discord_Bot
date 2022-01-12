use std::fs::OpenOptions;
use std::ops::Add;
use std::io::Write;
use chrono::{Duration, Local};
use csv_async::AsyncWriterBuilder;
use serenity::framework::standard::{CommandResult, Args};
use serenity::framework::standard::macros::command;
use serenity::model::prelude::*;
use serenity::prelude::*;
use thirtyfour::{DesiredCapabilities, Proxy, Capabilities, WebDriver};
use thirtyfour::error::WebDriverError;

use crate::consts::{CHROME_BINARY, USER_FILE};
use crate::discord::{UserRecord, UserAuthInfo};
use crate::dropbox_api;
use crate::third_party::{BrowserMobProxy, Status, Selenium};
use crate::{consts::{PRIMARY_COLOR, USER_DATA}};

async fn launch_selenium(email: String, password: String, proxy: BrowserMobProxy) -> Result<Status, WebDriverError> {
        proxy.create_proxy().await?;

    let proxy_port = proxy.get_proxy().await?;
	let index = proxy_port.proxyList.len() - 1;
    
    let mut caps = DesiredCapabilities::chrome();
    caps.set_proxy(Proxy::Manual {
        ftp_proxy: None, 
        http_proxy: Some(format!("http://{}:{}", proxy.host, proxy_port.proxyList[index].port)), 
        ssl_proxy: Some(format!("http://{}:{}", proxy.host, proxy_port.proxyList[index].port)),
        socks_proxy: None,
        socks_version: None,
        socks_username: None,
        socks_password: None,
        no_proxy: None
    })?;
    caps.accept_ssl_certs(true)?;
    caps.set_binary(CHROME_BINARY.lock().await.as_str())?;
    caps.add_chrome_arg("--proxy-server=http://localhost:8083")?;
    caps.add_chrome_arg("--ignore-certificate-errors")?;
    caps.set_headless()?;
    
    let selenium = Selenium::init(WebDriver::new("http://localhost:4444", &caps).await?, email.clone(), password.clone());

    selenium.setup().await?;

    
    BrowserMobProxy::new_har(&proxy).await?;
    let is_valid = selenium.run().await?;
	
	selenium.quit().await?;

	Ok(is_valid)
}

async fn add_account(email: String, password: String, msg: &mut Message, ctx: &Context) -> CommandResult {
	let proxy = BrowserMobProxy {host: "localhost", port: 8082};

	let handle = tokio::task::spawn( async move {
		launch_selenium(email.clone(), password.clone(), proxy).await.unwrap()
	}).await?;

	let is_valid = handle; 

	match is_valid {
		Status::VALID => {
			if !USER_DATA.lock().await.contains_key(msg.author.id.as_u64()) {
				let har = BrowserMobProxy::get_har(&proxy).await?;
				let len = har["log"]["entries"].as_array().unwrap().len();
				let bearer_token = &har["log"]["entries"][len - 1]["request"]["headers"][6]["value"].to_string();
	
				let user_record = &UserRecord {
					member_id: *msg.author.id.as_u64(),
					auth: bearer_token[1..bearer_token.len()-1].to_string(),
					last_registered: Local::now()			
				};
				
				USER_DATA.lock().await.insert(user_record.member_id, UserAuthInfo{ auth: user_record.auth.clone(), last_registered: user_record.last_registered});
	
				let mut wtr = AsyncWriterBuilder::new()
					.has_headers(false)
					.create_serializer(vec![]);
				
				wtr.serialize(user_record).await?;
				
				let mut file = OpenOptions::new()
				.append(true)
				.open("user_data.csv")
				.unwrap();
				
				if let Err(err) = write!(file, "{}", String::from_utf8(wtr.into_inner().await?).unwrap()) {
					eprintln!("Error when writing to a file: {}", err);
				}

				let res = dropbox_api::upload_file(USER_FILE.to_string()).await?;
				println!("File upload status code: {}", res);
				
				msg.edit(&ctx, |m| {
					m.embed(|e| e
						.colour(PRIMARY_COLOR)
						.field("Account Registered", "Account successfully registered", false)
					)
				}).await?;

				proxy.delete_proxy().await?;
			}
		},
		Status::INVALID => {
			msg.edit(&ctx, |m| {
				m.embed(|e| e
					.colour(PRIMARY_COLOR)
					.field("Account is not valid", "Wrong email or password", false))
			}).await?;

			proxy.delete_proxy().await?;
		}
	}

	Ok(())
}

#[command]
#[only_in("dm")]
#[description("Add BINUS account to discord bot")]
#[usage("[email];[password]")]
#[num_args(2)]
async fn add(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
	let email = args.single::<String>().unwrap();
	let password = args.single::<String>().unwrap();

	if USER_DATA.lock().await.contains_key(msg.author.id.as_u64()){
		let jwt_exp = USER_DATA.lock().await.get(msg.author.id.as_u64()).unwrap().last_registered.add(Duration::weeks(52));
		let now = chrono::offset::Local::now();
		if jwt_exp < now {
			let mut bot_msg = msg.channel_id.send_message(&ctx, |m| {
				m.embed(|e| e
					.colour(PRIMARY_COLOR)
					.field("Registering...", "Please wait a few seconds", false))
			}).await?;

			add_account(email, password, &mut bot_msg, ctx).await.unwrap();
		} else {
			msg.author.dm(&ctx, |m| {
				m.embed(|e| e
					.colour(PRIMARY_COLOR)
					.field("You've already registered", format!("Please wait **{} days** to re-register your account", jwt_exp.signed_duration_since(now).num_days()), false))
			}).await?;
		}
	} else {
		let mut bot_msg = msg.channel_id.send_message(&ctx, |m| {
			m.embed(|e| e
				.colour(PRIMARY_COLOR)
				.field("Registering...", "Please wait a few seconds", false))
		}).await?;

		add_account(email, password, &mut bot_msg, ctx).await?;
	}

	Ok(())
}