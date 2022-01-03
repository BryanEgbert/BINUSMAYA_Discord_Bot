use thirtyfour::{
	Capabilities, 
	prelude::*, 
	common::capabilities::desiredcapabilities::Proxy, error::WebDriverError
};
use std::{
	collections::{
		HashMap, 
		HashSet
	}, 
	fs::{
		File, 
		OpenOptions, 
		read_to_string, 
		metadata
	}, 
	thread::{
		self, 
		sleep
	}, 
	io::Write, 
	path::Path, 
	ops::Add
};
use csv_async::{AsyncReaderBuilder, AsyncWriterBuilder};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Local, Duration};
use futures::{stream ,StreamExt, future};
use tokio::{sync::Mutex};
use serenity::{
    async_trait,
    model::prelude::*,
    prelude::*,
	http::Http,
	utils::Colour,
};
use serenity::framework::standard::{
    StandardFramework,
    CommandResult,
	Args,
	help_commands,
	HelpOptions,
	CommandGroup,
	CommandError,
    macros::{
        command,
        group,
		help,
		hook
    },
};
use crate::third_party::*;
use crate::binusmaya::*;
use std::env;


const USER_FILE: &str = "user_data.csv";
const LOGIN_FILE: &str = "last_login.txt";
const PRIMARY_COLOR: Colour = Colour::BLUE;
lazy_static!{
	static ref USER_DATA: Mutex<HashMap<u64, UserAuthInfo>> = Mutex::new(HashMap::new());
	static ref CHROME_BINARY: Mutex<Option<String>> = Mutex::new(None);
}

#[derive(Serialize, Deserialize, Clone)]
struct UserRecord {
	member_id: u64,
	auth: String,
	last_registered: DateTime<Local>,
}
#[derive(Debug)]
struct UserAuthInfo {
	auth: String,
	last_registered: DateTime<Local>
}

#[group]
#[commands(ping, register, add, schedule, details, classes)]
pub struct General;

pub struct Handler;

async fn send_schedule_daily(ctx: &Context) {
	loop {
		let metadata =  metadata(LOGIN_FILE).unwrap();
	
		if let Ok(time) = metadata.modified() {
			let last_login = DateTime::<Local>::from(time).date();
			if last_login.succ().eq(&chrono::offset::Local::now().date()) {
				stream::iter(USER_DATA.lock().await.iter())
					.for_each_concurrent(8, |(user_id, user_auth_info)| async move {
						let context = ctx.clone();
						let binusmaya_api = BinusmayaAPI{token: user_auth_info.auth.to_string()};
						let schedule = binusmaya_api.get_schedule(&chrono::offset::Local::now().format("%Y-%-m-%-d").to_string()).await.unwrap();
						let channel_id = UserId(*user_id).create_dm_channel(&context.http)
							.await.unwrap().id;
		
						if let Some(classes) = schedule {
							ChannelId(*channel_id.as_u64()).send_message(&context.http, |m| {
								m.embed(|e| e
									.title("Today's Schedule")
									.description(format!("{} Sessions\n{}For more information about the topics/resources of the session, use `=details` command", classes.schedule.len(), classes))
									.colour(PRIMARY_COLOR)
								)
							}).await.unwrap();
		
							stream::iter(classes.schedule)
								.for_each_concurrent(8, |s| async {
									binusmaya_api.update_student_progress(s.custom_param.class_session_id).await.unwrap();
								}).await;
						} else {
							ChannelId(*channel_id.as_u64()).send_message(&context.http, |m| {
								m.embed(|e| e
									.title("Today's Schedule")
									.colour(PRIMARY_COLOR)
									.field("Holiday!", "No classes/sessions for today", true)
								)
							}).await.unwrap();
						}
					}).await;

				File::create(LOGIN_FILE).unwrap_or_else(|e| {
					panic!("Error in creating file: {:?}", e);
				});
			} else {
				sleep(Duration::seconds(1).to_std().unwrap()); // Sleep for one day
			}
		} else {
			panic!("File metadata not supported in your platform");
		}
	}
}

#[async_trait]
impl EventHandler for Handler {
	async fn ready(&self, ctx: Context, data_about_bot: Ready) {

		if !Path::new(USER_FILE).exists() {
			File::create(USER_FILE).unwrap_or_else(|e| {
				panic!("Error in creating file: {:?}", e);
			});	
		}

		if !Path::new(LOGIN_FILE).exists() {
			File::create(LOGIN_FILE).unwrap_or_else(|e| {
				panic!("Error in creating file: {:?}", e);
			});
		}
		
		let content = read_to_string(USER_FILE).expect("Something's wrong when reading a file");
		let rdr = AsyncReaderBuilder::new()
			.has_headers(false)
			.create_deserializer(content.as_bytes());
		let mut records = rdr.into_deserialize::<UserRecord>();
		while let Some(record) = records.next().await {
			let record = record.unwrap();
			USER_DATA.lock().await.insert(record.member_id, UserAuthInfo { auth: record.auth, last_registered: record.last_registered });
		}

		tokio::spawn(async move {
			println!("{:?} is running", thread::current().id());
			send_schedule_daily(&ctx).await;
		});

		println!("{} is ready", data_about_bot.user.name);
	}
}


#[help]
async fn help(ctx: &Context, msg:&Message, args: Args, help_options: &'static HelpOptions, groups: &[&'static CommandGroup], owners: HashSet<UserId>) -> CommandResult {
	let _ = help_commands::with_embeds(ctx, msg, args, help_options, groups, owners).await;

	Ok(())
}

#[hook]
async fn after_hook(_: &Context, _: &Message, cmd_name: &str, error: Result<(), CommandError>) {
    //  Print out an error if it happened
    if let Err(why) = error {
        println!("Error in {}: {:?}", cmd_name, why);
    }
}

pub async fn run() {
	*CHROME_BINARY.lock().await = env::args().nth(1);
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
			.delimiter(';')
			.prefix("=")
			.owners(owners))
			.after(after_hook)
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
			.colour(PRIMARY_COLOR)
			.title("reply to command")
			.field("test", "pong", false))
	}).await?;

	Ok(())
}

#[command]
#[description("Receive a DM to register your binus account for additional features")]
#[only_in("guild")]
async fn register(ctx: &Context, msg: &Message) -> CommandResult {
	msg.author.dm(&ctx, |m| {
		m.embed(|e| e
			.colour(PRIMARY_COLOR)
			.field("Register", "Please enter your BINUS email and password, e.g. `=add [email];[password]`", false))
	}).await?;
	Ok(())
}

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
    caps.set_binary(CHROME_BINARY.lock().await.clone().unwrap().as_str())?;
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

async fn add_account(email: String, password: String, msg: &Message, ctx: &Context) -> CommandResult {
	println!("starting");
	let proxy = BrowserMobProxy {host: "localhost", port: 8082};

	let handle = tokio::task::spawn( async move {
		launch_selenium(email.clone(), password.clone(), proxy).await.unwrap()
	}).await.expect("lol");

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
				
				msg.author.dm(&ctx, |m| {
					m.embed(|e| e
						.colour(PRIMARY_COLOR)
						.field("Account Registered", "Account successfully registered", false)
					)
				}).await?;

				proxy.delete_proxy().await?;
			}
		},
		Status::INVALID => {
			msg.author.dm(&ctx, |m| {
				m.embed(|e| e
					.colour(PRIMARY_COLOR)
					.field("Account is not valid", "Wrong email or password", false))
			}).await?;
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
			msg.channel_id.send_message(&ctx, |m| {
				m.embed(|e| e
					.colour(PRIMARY_COLOR)
					.field("Registering...", "Please wait a few seconds", false))
			}).await?;

			add_account(email, password, msg, ctx).await.unwrap();
		} else {
			msg.author.dm(&ctx, |m| {
				m.embed(|e| e
					.colour(PRIMARY_COLOR)
					.field("You've already registered", format!("Please wait **{} days** to re-register your account", jwt_exp.signed_duration_since(now).num_days()), false))
			}).await?;
		}
	} else {
		msg.author.dm(&ctx, |m| {
			m.embed(|e| e
				.colour(PRIMARY_COLOR)
				.field("Registering...", "Please wait a few seconds", false))
		}).await?;

		add_account(email, password, msg, ctx).await?;
	}

	Ok(())
}

#[command]
#[description("Get schedule")]
#[num_args(1)]
#[description("Get the schedule of the given date")]
#[usage("[YYYY-MM-DD]")]
#[example("2022-01-05")]
async fn schedule(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
	let date = args.single::<String>().unwrap();
	let user_data = USER_DATA.lock().await;

	if user_data.contains_key(msg.author.id.as_u64()) {
		let jwt_exp = user_data.get(msg.author.id.as_u64()).unwrap().last_registered.add(Duration::weeks(52));
		let now = chrono::offset::Local::now();
		if jwt_exp > now {
			let binusmaya_api = BinusmayaAPI{token: user_data.get(msg.author.id.as_u64()).unwrap().auth.clone()};
			let schedule = binusmaya_api.get_schedule(&date).await?;
	
			if let Some(class) = schedule {
				msg.channel_id.send_message(&ctx.http, |m| {
					m.embed(|e| e
						.title(format!("Schedule for {}", date.clone()))
						.description(format!("**{} Session(s)**\n{}", class.schedule.len(), class))
						.colour(PRIMARY_COLOR)
					)
				}).await?;
			} else {
				msg.channel_id.send_message(&ctx.http, |m| {
					m.embed(|e| e
						.title("Today's Schedule")
						.colour(PRIMARY_COLOR)
						.field("Holiday!", "No classes/sessions for today", true)
					)
				}).await?;
			}
		} else {
			msg.channel_id.send_message(&ctx.http, |m| {
				m.embed(|e| e
					.colour(PRIMARY_COLOR)
					.field("Your bearer token has expired", "please re-register using `=add` command", false)
				)
			}).await?;
		}
	} else {
		msg.channel_id.send_message(&ctx.http, |m| {
			m.embed(|e| e
				.colour(PRIMARY_COLOR)
				.field("You're not registered", "please register first using `=register` command", false)
			)
		}).await?;
	}

	Ok(())
}

#[command]
#[num_args(3)]
#[aliases("resource", "d")]
#[description("Get the subtopics and resources/article of the session")]
#[usage("[subject name];[Class component];[Session number]")]
#[example("Linear;LEC;1")]
async fn details(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
	let course_name = args.single::<String>()?;
	let class_component = args.single::<String>()?;
	let session_number = args.single::<usize>()?;

	let user_data = USER_DATA.lock().await;

	if user_data.contains_key(msg.author.id.as_u64()) {
		let jwt_exp = user_data.get(msg.author.id.as_u64()).unwrap().last_registered.add(Duration::weeks(52));
		let now = chrono::offset::Local::now();
		if jwt_exp > now {
			let binusmaya_api = BinusmayaAPI{token: user_data.get(msg.author.id.as_u64()).unwrap().auth.clone()};
	
			let class = stream::iter(binusmaya_api.get_classes().await?.classes)
				.filter(|c| future::ready(c.course_name.contains(&course_name) && c.ssr_component.eq(&class_component)))
				.next().await;
	
			if let Some(c) = class {
				let class_id = c.class_id;
				let class_details = binusmaya_api.get_class_details(class_id.clone()).await?;
		
				if class_details.sessions.len() < session_number {
					msg.channel_id.send_message(&ctx.http, |m| {
						m.embed(|e| e
							.colour(PRIMARY_COLOR)
							.field(format!("Session {} doesn't exists", session_number), format!("There's only {} Sessions", class_details.sessions.len()), false)
						)
					}).await.unwrap();
				} else {
					let session_id = &class_details.sessions[session_number - 1].id;
					let session_details = binusmaya_api.get_resource(session_id.to_string()).await.unwrap();
					msg.channel_id.send_message(&ctx.http, |m| {
						m.embed(|e| e
							.title(format!("{}\nSession {}", session_details.topic, session_details.session_number))
							.description(format!("**Subtopics**\n{:#?}\n\n**Resources**\n{}", session_details.course_sub_topic, session_details.resources))
							.colour(PRIMARY_COLOR)
							.url(format!("https://newbinusmaya.binus.ac.id/lms/course/{}/session/{}", class_id.clone(), session_id))
						)
					}).await?;
				}
			} else {
				msg.channel_id.send_message(&ctx.http, |m| {
					m.embed(|e| e
						.colour(PRIMARY_COLOR)
						.field(format!("subject named {} doesn't exists", course_name), "**Tips:** [subject name] and [class component] are case sensitive", false)
					)
				}).await?;
			}
		} else {
			msg.channel_id.send_message(&ctx.http, |m| {
				m.embed(|e| e
					.colour(PRIMARY_COLOR)
					.field("Your bearer token has expired", "please re-register using `=add` command", false)
				)
			}).await?;
		}	
	} else {
		msg.channel_id.send_message(&ctx.http, |m| {
			m.embed(|e| e
				.colour(PRIMARY_COLOR)
				.field("You're not registered", "please register first using `=register` command", false)
			)
		}).await?;
	}

	Ok(())
}

#[command]
#[aliases("c")]
#[description("Get the list of classes in your major")]
async fn classes(ctx: &Context, msg: &Message) -> CommandResult {
	let user_data = USER_DATA.lock().await;

	if user_data.contains_key(msg.author.id.as_u64()) {
		let jwt_exp = user_data.get(msg.author.id.as_u64()).unwrap().last_registered.add(Duration::weeks(52));
		let now = chrono::offset::Local::now();
		if jwt_exp > now {
			let binusmaya_api = BinusmayaAPI{token: user_data.get(msg.author.id.as_u64()).unwrap().auth.clone()};
			let classes = binusmaya_api.get_classes().await?;
	
			msg.channel_id.send_message(&ctx.http, |m| {
				m.embed(|e| e
					.title("Class List")
					.description(classes)
					.colour(PRIMARY_COLOR)
				)
			}).await?;
		} else {
			msg.channel_id.send_message(&ctx.http, |m| {
				m.embed(|e| e
					.colour(PRIMARY_COLOR)
					.field("Your bearer token has expired", "please re-register using `=add` command", false)
				)
			}).await?;
		}
	} else {
		msg.channel_id.send_message(&ctx.http, |m| {
			m.embed(|e| e
				.colour(PRIMARY_COLOR)
				.field("You're not registered", "please register first using `=register` command", false)
			)
		}).await?;
	}
	Ok(())
}


