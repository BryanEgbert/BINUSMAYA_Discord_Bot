use serenity::model::prelude::Ready;
use thirtyfour::{Capabilities, prelude::*, common::capabilities::desiredcapabilities::Proxy};
use serenity::http::Http;
use serenity::model::prelude::UserId;
use std::{collections::{HashMap, HashSet}, fs::{File, OpenOptions, read_to_string}, io::Write, path::Path};
use csv_async::{AsyncSerializer, AsyncDeserializer};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Local};
use tokio_stream::StreamExt;
use tokio::sync::Mutex;
use serenity::{
    async_trait,
    model::{channel::Message},
    prelude::*,
};
use serenity::framework::standard::{
    StandardFramework,
    CommandResult,
	Args,
	help_commands,
	HelpOptions,
	CommandGroup,
    macros::{
        command,
        group,
		help
    }
};


use crate::prelude::*;
pub use crate::binusmaya::*;

use std::env;

const FILE_NAME: &str = "user_data.csv";
lazy_static!{
	static ref USER_DATA: Mutex<HashMap<u64, String>> = Mutex::new(HashMap::new());
}

#[derive(Serialize, Deserialize, Clone)]
struct UserRecord {
	member_id: u64,
	auth: String,
	last_registered: DateTime<Local>,
}


#[group]
#[commands(ping, register, add, schedule)]
pub struct General;

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {

	async fn ready(&self, _ctx: Context, data_about_bot: Ready) {
		println!("{} is ready", data_about_bot.application.id);

		if !Path::new(FILE_NAME).exists() {
			File::create("user_data.csv").unwrap_or_else(|e| {
				panic!("Error in creating file: {:?}", e);
			});	
		}

		let content = read_to_string(FILE_NAME).expect("Something's wrong when reading a file");
		let mut rdr = AsyncDeserializer::from_reader(content.as_bytes());
		let mut records = rdr.deserialize::<UserRecord>();
		while let Some(record) = records.next().await {
			let record = record.unwrap();
			USER_DATA.lock().await.insert(record.member_id, record.auth);
		}
	}
}


#[help]
#[suggestion_text(register)]
async fn help(ctx: &Context, msg:&Message, args: Args, help_options: &'static HelpOptions, groups: &[&'static CommandGroup], owners: HashSet<UserId>) -> CommandResult {
	let _ = help_commands::with_embeds(ctx, msg, args, help_options, groups, owners).await;

	Ok(())
}

pub async fn run() {
	let token = env::var("DISCORD_TOKEN").expect("invalid token");
	let http = Http::new_with_token(&token);
	let (owners, _) = match http.get_current_application_info().await {
		Ok(info) => {
			let mut owners = HashSet::new();
			if let Some(team) = info.team {
				owners.insert(team.owner_user_id);
			} else {
				owners.insert(info.owner.id);
			}
			match http.get_current_user().await {
				Ok(bot_id) => (owners, bot_id.id),
				Err(e) => panic!("Couldn't get bot id: {:?}", e),
			}
		},
		Err(e) => panic!("Couldn't get app info: {:?}", e)
	};
	let framework = StandardFramework::new()
		.configure(|c| c
			.prefix("=")
			.owners(owners))
		.group(&GENERAL_GROUP)
		.help(&HELP);

	let mut client = Client::builder(token)
		.event_handler(Handler)
		.framework(framework)
		.await
		.expect("Error in creating bot");

	if let Err(e) = client.start().await {
		println!("An error has occured: {:?}", e);
	}
}

#[command]
#[description = "send pong!"]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
	msg.channel_id.send_message(&ctx.http, |m| {
		m.embed(|e| e
			.colour(0x03aaf9)
			.title("reply to command")
			.field("test", "pong", false))
	}).await?;

	Ok(())
}

#[command]
#[description("Receive a DM to register your binus account for additional features which is still under development")]
#[only_in("guild")]
async fn register(ctx: &Context, msg: &Message) -> CommandResult {
	msg.author.dm(&ctx, |m| {
		m.embed(|e| e
			.colour(0x03aaf9)
			.field("Register", "Please enter your BINUS email and password, e.g. `=add [email] [password]`", false))
	}).await?;
	Ok(())
}

#[command]
#[only_in("dm")]
#[num_args(2)]
#[help_available(false)]
async fn add(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
	let email = args.single::<String>().unwrap();
	let password = args.single::<String>().unwrap();

	msg.author.dm(&ctx, |m| {
		m.embed(|e| e
			.colour(0x03aaf9)
			.field("Registering...", "Please wait a few seconds", false))
	}).await?;

	let proxy = BrowserMobProxy {host: "localhost", port: 8082, path: "./browsermob-proxy-2.1.4/bin/browsermob-proxy"};
    
    proxy.create_proxy().await?;

    let proxy_port = proxy.get_proxy().await?;
    
    let mut caps = DesiredCapabilities::chrome();
    caps.set_proxy(Proxy::Manual {
        ftp_proxy: None, 
        http_proxy: Some(format!("http://{}:{}", proxy.host, proxy_port)), 
        ssl_proxy: Some(format!("http://{}:{}", proxy.host, proxy_port)),
        socks_proxy: None,
        socks_version: None,
        socks_username: None,
        socks_password: None,
        no_proxy: None
    })?;
    caps.accept_ssl_certs(true)?;
    caps.set_binary("/usr/bin/google-chrome")?;
    caps.add_chrome_arg("--proxy-server=http://localhost:8083")?;
    caps.add_chrome_arg("--ignore-certificate-errors")?;
    caps.set_headless()?;
    
    let selenium = Selenium::init(WebDriver::new("http://localhost:4444", &caps).await?, email.clone(), password.clone());

    selenium.setup().await?;

    
    BrowserMobProxy::new_har(&proxy).await?;
    let is_valid = selenium.run().await?;
	
	selenium.quit().await?;

	match is_valid {
		Status::VALID => {
			let har = BrowserMobProxy::get_har(&proxy).await?;
			let len = har["log"]["entries"].as_array().unwrap().len();
			let bearer_token = &har["log"]["entries"][len - 1]["request"]["headers"][6]["value"].to_string();

			let user_record = &UserRecord {
				member_id: *msg.author.id.as_u64(),
				auth: bearer_token[1..bearer_token.len()-1].to_string(),
				last_registered: Local::now()			
			};

			let mut wtr = AsyncSerializer::from_writer(vec![]);

			wtr.serialize(user_record).await?;

			let mut file = OpenOptions::new()
			.append(true)
			.open("user_data.csv")
			.unwrap();

			if let Err(err) = write!(file, "{}", String::from_utf8(wtr.into_inner().await?).unwrap()) {
				eprintln!("Error when writing to a file: {}", err);
			}
			
			msg.author.dm(&ctx, |m| {
				m.embed(|e| e
					.colour(0x03aaf9)
					.field("Account Registered", "Account successfully registered", false)
			)}).await?;
		},
		Status::INVALID => {
			msg.author.dm(&ctx, |m| {
				m.embed(|e| e
					.colour(0x03aaf9)
					.field("Account is not valid", "Wrong email or password", false))
			}).await?;
		}
	}

	Ok(())
}

#[command]
async fn schedule(ctx: &Context, msg: &Message) -> CommandResult {
	println!("lol");

	let user_data = USER_DATA.lock().await;

	if user_data.contains_key(msg.author.id.as_u64()) {
		let binusmaya_api = BinusmayaAPI{token: user_data.get(msg.author.id.as_u64()).unwrap().clone()};
		let schedule = binusmaya_api.get_schedule().await.expect("something's wrong");
		// msg.channel_id.send_message(&ctx.http, |m| {
		// 	m.embed(|e| e
		// 		.colour(0x03aaf9)
		// 		.field("Schedule", schedule, false)
		// 	)
		// }).await.unwrap();
	} else {
		msg.channel_id.send_message(&ctx.http, |m| {
			m.embed(|e| e
				.colour(0x03aaf9)
				.field("You're not registered", "please register first using `=register` command", false)
			)
		}).await.unwrap();
	}

	println!("{}", chrono::offset::Local::now().format("%Y-%m-1"));
	
	Ok(())
}


