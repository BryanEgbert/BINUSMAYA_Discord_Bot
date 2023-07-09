use chrono::{DateTime, Duration, Local, NaiveDate};

use futures::{stream, StreamExt};
use serde::{Deserialize, Serialize};
use serenity::{framework::standard::{
    help_commands,
    macros::{group, help, hook},
    Args, CommandError, CommandGroup, CommandResult, HelpOptions, StandardFramework,
}, utils::MessageBuilder};
use serenity::{
    async_trait, client::bridge::gateway::ShardManager, http::Http, model::prelude::*, prelude::*
};
use std::{
    collections::HashSet,
    fs::{metadata, File},
    sync::Arc,
    thread::{self, sleep},
};
use tokio::fs::{write, File as TokioFile};

use crate::{discord::{commands::{
    general::{
        about::*, add::*, ping::*, register::*
    },
    new_binusmaya::{
        announcement::*, classes::*, session::*, ongoing::*, 
        schedule::*, upcoming::*,
    },
    old_binusmaya::{
        sat::*, comserv::*, assignment::*,
    }
}, helper::update_cookie_all}, consts::{OLDBINUSMAYA_USER_FILE, LOGIN_FILE, NEWBINUSMAYA_USER_FILE, self}, api::{new_binusmaya_api::*, self}};

use std::env;

const WIFI_ATTENDANCE: &str = "Wifi Attendance";
const VIRTUAL_CLASS: &str = "Virtual Class";
const FORUM: &str = "Forum";
const GSLC: &str = "GSLC";

#[derive(Serialize, Deserialize, Clone)]
pub struct NewBinusmayaUserRecord {
    pub member_id: u64,
    pub auth: String,
    pub last_registered: DateTime<Local>,
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
#[summary("Commands that fetch data from new binusmaya")]
#[commands(schedule, session, classes, ongoing, upcoming, announcement)]
pub struct NewBinusmaya;

#[group]
#[summary("Commands that fetch data from old binusmaya")]
#[commands(sat, comserv, assignment)]
pub struct OldBinusmaya;

pub struct Handler;

fn start_third_party_apps() {
    // Command::new("./chromedriver")
    //     .arg("--port=9222")
    //     .spawn()
    //     .expect("Failed to run chrome driver");

    // Command::new("sh")
    //     .args([
    //         "./browsermob-proxy-2.1.4/bin/browsermob-proxy",
    //         "--address",
    //         "localhost",
    //         "--port",
    //         "8082",
    //     ])
    //     .spawn()
    //     .expect("Failed to start browsermob-proxy");
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

async fn update_student_progress(new_binusmaya_api: &NewBinusmayaAPI, schedule_details: ScheduleDetails) {
    let class_session = new_binusmaya_api
        .get_resource(schedule_details.custom_param.class_session_id.to_string())
        .await
        .unwrap();
    for resource in class_session.resources.list {
        if !resource.resource_type.eq(VIRTUAL_CLASS)
            && !resource.resource_type.eq(FORUM)
            && !resource.resource_type.eq(WIFI_ATTENDANCE)
        {
            new_binusmaya_api
                .update_student_progress(&resource.id, resource.class_session_id)
                .await
                .unwrap();
        }
    }
} 

async fn post_forum_reminder(ctx: &Context, new_binusmaya_api: &NewBinusmayaAPI, schedule_details: ScheduleDetails, member_id: &u64) -> Result<(), chrono::format::ParseError> {
    let now = NaiveDate::parse_from_str(chrono::offset::Local::now().format("%FT%X").to_string().as_str(),"%FT%X")?;
    let date_end = NaiveDate::parse_from_str(schedule_details.date_start.as_str(), "%FT%X")?.checked_sub_signed(Duration::days(7)).unwrap();

    if schedule_details.class_delivery_mode.eq(GSLC) && now.eq(&date_end) {
        let class_session = new_binusmaya_api
            .get_resource(schedule_details.custom_param.class_session_id.to_string())
            .await.unwrap();
        let private_channel = UserId(*member_id).create_dm_channel(&ctx.http).await;

        if let Ok(channel) = private_channel {
            let mut content = MessageBuilder::new();
            content.push_bold_line("Don't forget to post a forum, today's the deadline.");
            
            class_session.resources.list.iter().for_each(|r| {
                if r.resource_type.eq(FORUM) && r.progress_status != 2 {
                    content.push_quote_line(format!("**{} - Session {}**", schedule_details.content, schedule_details.custom_param.session_number))
                        .push_quote_line(format!("[forum link](https://newbinusmaya.binus.ac.id/lms/course/{}/forum/{})", schedule_details.custom_param.class_id, schedule_details.custom_param.class_session_id));
                } 
            });
    
            channel.id.send_message(&ctx.http, |m| {
                m.embed(|e| e
                    .title("Post GSLC Forum Reminder")
                    .description(content.build())
    
                )
            }).await.unwrap();
        }
    }

    Ok(())
}

async fn loop_student_schedule(_ctx: &Context) {
    let user_data = consts::NEW_BINUSMAYA_REPO.get_all().unwrap();

    stream::iter(user_data)
        .for_each_concurrent(8, |user| async move {
            let binusmaya_api = NewBinusmayaAPI {
                token: user.auth.to_string(),
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
                    // let schedule_details = s.clone();
                    
                    update_student_progress(&binusmaya_api, s).await;
                    // post_forum_reminder(ctx, &binusmaya_api, schedule_details, &user.member_id).await.unwrap();
                })
                .await;
            }
        })
        .await;
}

async fn daily_event(ctx: &Context) {
    loop {
        let metadata = metadata(LOGIN_FILE).unwrap();

        if let Ok(time) = metadata.modified() {
            let last_login = DateTime::<Local>::from(time).date_naive();
            if last_login.succ_opt().unwrap().eq(&chrono::offset::Local::now().date_naive()) {
                loop_student_schedule(ctx).await;

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
        // TODO: change to intialize sqlite instead
        // fetch_file().await;
        // start_third_party_apps();
        
        // let newbinusmaya_content = read_to_string(NEWBINUSMAYA_USER_FILE).expect("Something's wrong when reading a file");


        // let rdr = AsyncReaderBuilder::new()
        //     .has_headers(false)
        //     .create_deserializer(newbinusmaya_content.as_bytes());

        // let mut records = rdr.into_deserialize::<models::user::NewBinusmayaUser>();
        // while let Some(record) = records.next().await {
        //     let record = record.unwrap();
        //     NEWBINUSMAYA_USER_DATA.lock().await.insert(
        //         record.member_id,
        //         NewBinusmayaUserAuthInfo {
        //             auth: record.auth,
        //             last_registered: record.last_registered,
        //         },
        //     );
        // }

        update_cookie_all().await;

        tokio::spawn(async move {
            println!("{:?} is running", thread::current().id());
            daily_event(&ctx).await;
        });

        println!("{} is ready", data_about_bot.user.name);
    }
}

#[help]
#[strikethrough_commands_tip_in_guild("")]
#[strikethrough_commands_tip_in_dm("")]
#[embed_success_colour("#3498DB")]
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
async fn before(ctx: &Context, msg: &Message, _cmd_name: &str) -> bool {
    msg.react(&ctx, '👍').await.unwrap();

    true
}

#[hook]
async fn unknown_command(ctx: &Context, msg: &Message, cmd_name: &str) {
    msg.channel_id.send_message(&ctx, |m| {
        m.embed(|e| e
            .field("Couldn't find command name", format!("Couldn't find command named **{}**, use `=help` command to see the command list", cmd_name), false))
    }).await.unwrap();
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

    let http = Http::new(&token);

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
        .configure(|c| c.delimiter(';').prefix("!").owners(owners))
        .before(before)
        .after(after_hook)
        .unrecognised_command(unknown_command)
        .group(&GENERAL_GROUP)
        .group(&NEWBINUSMAYA_GROUP)
        .group(&OLDBINUSMAYA_GROUP)
        .help(&HELP);

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(&token, intents)
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

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn post_reminder_test() {

        let binusmaya_api = NewBinusmayaAPI {
            token: "Bearer eyJhbGciOiJQUzI1NiIsInR5cCI6IkpXVCJ9.eyJodHRwOi8vc2NoZW1hcy54bWxzb2FwLm9yZy93cy8yMDA1LzA1L2lkZW50aXR5L2NsYWltcy9uYW1laWRlbnRpZmllciI6ImJjOTg4NTM5LTg5ZWQtNDA5OS04MTkzLWNiMGUzNTFjMjg1NCIsImh0dHA6Ly9zY2hlbWFzLnhtbHNvYXAub3JnL3dzLzIwMDUvMDUvaWRlbnRpdHkvY2xhaW1zL25hbWUiOiJCUllBTiBFR0JFUlQiLCJodHRwOi8vc2NoZW1hcy5taWNyb3NvZnQuY29tL3dzLzIwMDgvMDYvaWRlbnRpdHkvY2xhaW1zL3JvbGUiOiJHdWVzdCIsIm5iZiI6MTY0NDIwNzUyNSwiZXhwIjoxNjc1NzQzNTI1LCJpc3MiOiJCaW51c1NlcnZpY2VzIiwiYXVkIjoiTmV4dXMuSWRlbnRpdHlTZXJ2aWNlIn0.0GMmYGQh7HEQdwPwZmWaxtt4CvtF3Ke9LwWaA4w8HbX-5PtewpvbNHKnzmv68_nu6UqDQPMugifLeNQ3wQaSl9IIURw9BfBTegksIzHgDtcGV4U3dd7Pc2MFu4YuHXuhnF0bCu0DxSeDs5qfGnKbK8s6N6VKirQO360uKy0_Lv8".to_string(),
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

        let mut content = MessageBuilder::new();
        content.push_underline_line("**Don't forget to post a forum, today's the deadline.**").push_quote_line("**EESE 2 - Session 1**\n[forum link](https://newbinusmaya.binus.ac.id/lms/course/a/forum/a)");

        println!("{:?}", content);

        if let Some(classes) = schedule {
            stream::iter(classes.schedule)
            .for_each_concurrent(8, |s| async move {
                let schedule_details = s.clone();
    

                
                let now = NaiveDate::parse_from_str(chrono::offset::Local::now().format("%FT%X").to_string().as_str(),"%FT%X").unwrap();
                let date_end = NaiveDate::parse_from_str(schedule_details.date_end.as_str(), "%FT%X").unwrap();
                println!("{}", s.content);
                println!("Date now: {:?}\nDate end: {:?}", now, date_end);
                println!("{:?}", now.eq(&date_end));
            })
            .await;
        }
    }
}