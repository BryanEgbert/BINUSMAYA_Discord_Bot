use std::{
	collections::HashSet, 
	fs::{
		File, 
		read_to_string, 
		metadata
	}, 
	thread::{
		self, 
		sleep
	}, 
	path::Path, sync::Arc
};
use csv_async::AsyncReaderBuilder;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Local, Duration, NaiveDate};
use futures::{stream ,StreamExt};
use serenity::{
    async_trait,
    model::prelude::*,
    prelude::*,
	http::Http, client::bridge::gateway::ShardManager,
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
        group,
		help,
		hook
    },
};

use crate::discord::commands::{
		ping::*, 
		register::*, 
		classes::*,
		details::*,
		schedule::*,
		add::*,
		ongoing::*,
		upcoming::*,
	};
use crate:: {
	consts::{
		USER_DATA, LOGIN_FILE, USER_FILE, 
	}, 
	binusmaya::*
};

use std::env;

#[derive(Serialize, Deserialize, Clone)]
pub struct UserRecord {
	pub member_id: u64,
	pub auth: String,
	pub last_registered: DateTime<Local>,
}

#[derive(Debug)]
pub struct UserAuthInfo {
	pub auth: String,
	pub last_registered: DateTime<Local>
}

pub struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
	type Value = Arc<Mutex<ShardManager>>;
}

#[group]
#[commands(ping, register, add)]
pub struct General;

#[group]
#[commands(schedule, details, classes, ongoing, upcoming)]
pub struct Binus;

pub struct Handler;

async fn update_student_progress_daily() {
	loop {
		let metadata =  metadata(LOGIN_FILE).unwrap();
	
		if let Ok(time) = metadata.modified() {
			let last_login = DateTime::<Local>::from(time).date();
			if last_login.succ().eq(&chrono::offset::Local::now().date()) {
				stream::iter(USER_DATA.lock().await.iter())
					.for_each_concurrent(8, |(member_id, user_auth_info)| async move {
						println!("Updating student progress for {}", member_id);

						let binusmaya_api = BinusmayaAPI{token: user_auth_info.auth.to_string()};
						let schedule = binusmaya_api.get_schedule(&NaiveDate::parse_from_str(chrono::offset::Local::now().format("%Y-%-m-%-d").to_string().as_str(), "%Y-%-m-%-d").unwrap()).await.unwrap();
		
						if let Some(classes) = schedule {
							stream::iter(classes.schedule)
								.for_each_concurrent(8, |s| async {
									let class_session = binusmaya_api.get_resource(s.custom_param.class_session_id).await.unwrap();
									for resource in class_session.resources.resources {
										if !resource.resource_type.eq("Virtual Class") && !resource.resource_type.eq("Forum") && !resource.resource_type.eq("Assignment") {
											binusmaya_api.update_student_progress(&resource.id).await.unwrap();
										}
									}
								}).await;
						}
					}).await;

				File::create(LOGIN_FILE).unwrap_or_else(|e| {
					panic!("Error in creating file: {:?}", e);
				});
			} else {
				sleep(Duration::seconds(1).to_std().unwrap());
			}
		} else {
			panic!("File metadata not supported in your platform");
		}
	}
}

#[async_trait]
impl EventHandler for Handler {
	async fn ready(&self, _ctx: Context, data_about_bot: Ready) {

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
			update_student_progress_daily().await;
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
	let token = env::var("DISCORD_TOKEN").expect("invalid token");
	let app_id: u64 = env::var("APPLICATION_ID").expect("Expected application id in env").parse().expect("Invalid application id");
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
			.prefix("!")
			.owners(owners))
			.after(after_hook)
			.group(&GENERAL_GROUP)
			.group(&BINUS_GROUP)
			.help(&HELP);

	let mut client = Client::builder(token)
		.event_handler(Handler)
		.framework(framework)
		.application_id(app_id)
		.await
		.expect("Error in creating bot");

	{
			
		let mut data = client.data.write().await;
		data.insert::<ShardManagerContainer>(Arc::clone(&client.shard_manager));
	}

	if let Err(e) = client.start().await {
		println!("An error has occured: {:?}", e);
	}
}