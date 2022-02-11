use chrono::{DateTime, Duration, Local, NaiveDate};
use csv_async::AsyncReaderBuilder;
use futures::{stream, StreamExt};
use serde::{Deserialize, Serialize};
use serenity::framework::standard::{
    help_commands,
    macros::{group, help, hook},
    Args, CommandError, CommandGroup, CommandResult, HelpOptions, StandardFramework,
};
use serenity::{
    async_trait, client::bridge::gateway::ShardManager, http::Http, model::prelude::*, prelude::*,
};
use std::{
    collections::HashSet,
    fs::{metadata, read_to_string, File},
    sync::Arc,
    thread::{self, sleep}, process::Command,
};
use tokio::fs::{write, File as TokioFile};

use crate::{discord::commands::{
    general::{
        about::*, add::*, ping::*, register::*
    },
    new_binusmaya::{
        announcement::*, classes::*, details::*, ongoing::*, 
        schedule::*, upcoming::*
    },
    old_binusmaya::{
        sat::*, comserv::*,
    }
}, consts::{OLDBINUSMAYA_USER_FILE, LOGIN_FILE, NEWBINUSMAYA_USER_DATA, NEWBINUSMAYA_USER_FILE}, api::{new_binusmaya_api::*, old_binusmaya_api::{BinusianData}, self}};

use std::env;

use super::helper::update_cookie;

#[derive(Serialize, Deserialize, Clone)]
pub struct NewBinusmayaUserRecord {
    pub member_id: u64,
    pub auth: String,
    pub last_registered: DateTime<Local>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UserCredential {
    pub email: String,
    pub password: String
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UserBinusianData {
    pub binusian_id: String,
    pub display_name: String,
    pub user_id: String,
    pub role_id: u8,
    pub specific_role_id: u8,
}

impl UserBinusianData {
    pub fn init_data(binusian_data: &BinusianData) -> Self {
        let user_binusian_data = UserBinusianData {
            binusian_id: binusian_data.binusian_id.clone(),
            display_name: format!("{} {}", binusian_data.first_name, binusian_data.last_name),
            user_id: binusian_data.nim.clone(),
            role_id: 2,
            specific_role_id: 104,
        };

        user_binusian_data
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OldBinusmayaUserRecord {
    pub member_id: u64,
    pub user_credential: UserCredential,
    pub binusian_data: UserBinusianData
}

pub struct NewBinusmayaUserAuthInfo {
    pub auth: String,
    pub last_registered: DateTime<Local>,
}

pub struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

#[group]
#[commands(ping, register, add, about)]
pub struct General;

#[group]
#[commands(schedule, details, classes, ongoing, upcoming, announcement)]
pub struct NewBinusmaya;

#[group]
#[commands(sat, comserv)]
pub struct OldBinusmaya;

pub struct Handler;

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
    TokioFile::create(LOGIN_FILE)
        .await
        .expect("Error in creating login.txt");

    TokioFile::create(NEWBINUSMAYA_USER_FILE).await.expect("Error in creating new binusmaya file");

    TokioFile::create(OLDBINUSMAYA_USER_FILE).await.expect("Error in creating old binusmaya file");
    

    let new_binusmaya_user_content = api::dropbox_api::download_file(NEWBINUSMAYA_USER_FILE.to_string())
        .await
        .unwrap();

    if let Some(content) = new_binusmaya_user_content {
        write(NEWBINUSMAYA_USER_FILE, content.as_bytes()).await.unwrap();
    }

    let old_binusmaya_user_content = api::dropbox_api::download_file(OLDBINUSMAYA_USER_FILE.to_string())
        .await
        .unwrap();

    if let Some(content) = old_binusmaya_user_content {
        write(OLDBINUSMAYA_USER_FILE, content.as_bytes()).await.unwrap();
    }
    
    println!("File created successfully");
}

async fn update_student_progress() {
    stream::iter(NEWBINUSMAYA_USER_DATA.lock().await.iter())
        .for_each_concurrent(8, |(member_id, user_auth_info)| async move {
            println!("Updating student progress for {}", member_id);

            let binusmaya_api = NewBinusmayaAPI {
                token: user_auth_info.auth.to_string(),
            };
            let schedule = binusmaya_api
                .get_schedule(
                    &NaiveDate::parse_from_str(
                        chrono::offset::Local::now()
                            .format("%Y-%-m-%-d")
                            .to_string()
                            .as_str(),
                        "%Y-%-m-%-d",
                    )
                    .unwrap(),
                )
                .await
                .unwrap();

            if let Some(classes) = schedule {
                stream::iter(classes.schedule)
                    .for_each_concurrent(8, |s| async {
                        let class_session = binusmaya_api
                            .get_resource(s.custom_param.class_session_id)
                            .await
                            .unwrap();
                        for resource in class_session.resources.resources {
                            if !resource.resource_type.eq("Virtual Class")
                                && !resource.resource_type.eq("Forum")
                                && !resource.resource_type.eq("Assignment")
                            {
                                binusmaya_api
                                    .update_student_progress(&resource.id)
                                    .await
                                    .unwrap();
                            }
                        }
                    })
                    .await;
            }
        })
        .await;
}

async fn daily_update() {
    loop {
        let metadata = metadata(LOGIN_FILE).unwrap();

        if let Ok(time) = metadata.modified() {
            let last_login = DateTime::<Local>::from(time).date();
            if last_login.succ().eq(&chrono::offset::Local::now().date()) {
                update_student_progress().await;
                update_cookie(None).await;

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
        fetch_file().await;
        start_third_party_apps();
        
        let newbinusmaya_content = read_to_string(NEWBINUSMAYA_USER_FILE).expect("Something's wrong when reading a file");

        let rdr = AsyncReaderBuilder::new()
            .has_headers(false)
            .create_deserializer(newbinusmaya_content.as_bytes());

        let mut records = rdr.into_deserialize::<NewBinusmayaUserRecord>();
        while let Some(record) = records.next().await {
            let record = record.unwrap();
            NEWBINUSMAYA_USER_DATA.lock().await.insert(
                record.member_id,
                NewBinusmayaUserAuthInfo {
                    auth: record.auth,
                    last_registered: record.last_registered,
                },
            );
        }

        update_cookie(None).await;

        tokio::spawn(async move {
            println!("{:?} is running", thread::current().id());
            daily_update().await;
        });

        println!("{} is ready", data_about_bot.user.name);
    }
}

#[help]
async fn help(
    ctx: &Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>,
) -> CommandResult {
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
    let _ = env::var("SECRET_KEY").expect("expect SECRET KEY in env var");

    let token = env::var("DISCORD_TOKEN").expect("invalid token");
    let app_id: u64 = env::var("APPLICATION_ID")
        .expect("Expected application id in env")
        .parse()
        .expect("Invalid application id");
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
        }
        Err(e) => panic!("Couldn't get app info: {:?}", e),
    };
    let framework = StandardFramework::new()
        .configure(|c| c.delimiter(';').prefix("=").owners(owners))
        .after(after_hook)
        .group(&GENERAL_GROUP)
        .group(&NEWBINUSMAYA_GROUP)
        .group(&OLDBINUSMAYA_GROUP)
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
