use chrono::{DateTime, Duration, Local, NaiveDate};
use csv_async::AsyncReaderBuilder;
use futures::{stream, StreamExt};
use serde::{Deserialize, Serialize};
use magic_crypt::MagicCryptTrait;
use serenity::framework::standard::{
    help_commands,
    macros::{group, help, hook},
    Args, CommandError, CommandGroup, CommandResult, HelpOptions, StandardFramework,
};
use serenity::{
    async_trait, client::bridge::gateway::ShardManager, http::Http, model::prelude::*, prelude::*,
};
use thirtyfour::{WebDriver, DesiredCapabilities};
use std::{
    collections::HashSet,
    fs::{metadata, read_to_string, File},
    sync::Arc,
    thread::{self, sleep},
};

use crate::{discord::commands::{
    about::*, add::*, announcement::*, classes::*, details::*, ongoing::*, ping::*, register::*,
    schedule::*, upcoming::*, test::*
}, consts::{OLDBINUSMAYA_USER_FILE, CHROME_SERVER_URL, OLD_BINUSMAYA, OLDBINUSMAYA_USER_DATA, LOGIN_FILE, NEWBINUSMAYA_USER_DATA, NEWBINUSMAYA_USER_FILE, MAGIC_CRYPT}, third_party::Selenium, binusmaya::*};

use std::env;

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

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OldBinusmayaUserRecord {
    pub member_id: u64,
    pub user_credential: UserCredential
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
#[commands(schedule, details, classes, ongoing, upcoming, announcement, test)]
pub struct Binus;

pub struct Handler;

async fn update_student_progress() {
    stream::iter(NEWBINUSMAYA_USER_DATA.lock().await.iter())
        .for_each_concurrent(8, |(member_id, user_auth_info)| async move {
            println!("Updating student progress for {}", member_id);

            let binusmaya_api = BinusmayaAPI {
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

async fn update_cookie() {
    let oldbinusmaya_content = read_to_string(OLDBINUSMAYA_USER_FILE).expect("Something's wrong when reading a file");

    let rdr = AsyncReaderBuilder::new()
        .has_headers(false)
        .create_deserializer(oldbinusmaya_content.as_bytes());

    let caps = DesiredCapabilities::chrome();
    let mut records = rdr.into_deserialize::<OldBinusmayaUserRecord>();
    while let Some(record) = records.next().await {
        let record = record.unwrap();
        println!("{:?}", record);
        let selenium = Selenium::init(WebDriver::new(CHROME_SERVER_URL, &caps).await.unwrap(), record.user_credential.email, MAGIC_CRYPT.decrypt_base64_to_string(record.user_credential.password).unwrap());
        let is_valid = selenium.run(&OLD_BINUSMAYA.to_string()).await;
        if let Ok(status) = is_valid {
            match status {
                crate::third_party::Status::VALID(_) => {
                    OLDBINUSMAYA_USER_DATA.lock().await.insert(record.member_id, selenium.get_cookie().await.unwrap());
                },
                crate::third_party::Status::INVALID(_) => {
                    continue;
                },
                crate::third_party::Status::ERROR(_) => {
                    continue;
                },
            }
        }
    }
}

async fn daily_update() {
    loop {
        let metadata = metadata(LOGIN_FILE).unwrap();

        if let Ok(time) = metadata.modified() {
            let last_login = DateTime::<Local>::from(time).date();
            if last_login.succ().eq(&chrono::offset::Local::now().date()) {
                update_student_progress().await;

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
