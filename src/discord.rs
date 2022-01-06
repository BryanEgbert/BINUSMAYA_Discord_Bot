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
use chrono::{DateTime, Local, Duration};
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

use crate::{
	commands::{
		ping::*, 
		register::*, 
		classes::*,
		details::*,
		schedule::*,
		add::*
	}, 
	consts::{
		PRIMARY_COLOR, 
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
									.description(format!("{} Sessions\n{}For more information about the topics, resources of the session and to get the link of the class, use `=details` command", classes.schedule.len(), classes))
									.colour(PRIMARY_COLOR)
								)
							}).await.unwrap();
		
							stream::iter(classes.schedule)
								.for_each_concurrent(8, |s| async {
									let class_session = binusmaya_api.get_resource(s.custom_param.class_session_id).await.unwrap();
									for resource in class_session.resources.resources {
										if !resource.resource_type.eq("Virtual Class") && !resource.resource_type.eq("Forum") && !resource.resource_type.eq("Assignment") {
											binusmaya_api.update_student_progress(&resource.id).await.unwrap();
										}
									}
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
				sleep(Duration::seconds(1).to_std().unwrap());
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

	{
			
		let mut data = client.data.write().await;
		data.insert::<ShardManagerContainer>(Arc::clone(&client.shard_manager));
	}

	if let Err(e) = client.start().await {
		println!("An error has occured: {:?}", e);
	}
}