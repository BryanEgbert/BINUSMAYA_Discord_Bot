use serenity::http::Http;
use serenity::model::prelude::UserId;
use std::collections::HashSet;
use serenity::{
    async_trait,
    model::{channel::Message, gateway::Ready},
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

use std::env;

#[group]
#[commands(ping, register)]
pub struct General;

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {}

#[help]
async fn help(ctx: &Context, msg:&Message, args: Args, help_options: &'static HelpOptions, groups: &[&'static CommandGroup], owners: HashSet<UserId>) -> CommandResult {
	let _ = help_commands::with_embeds(ctx, msg, args, help_options, groups, owners).await;

	Ok(())
}

pub async fn run() {
	let token = env::var("DISCORD_TOKEN").expect("invalid token");
	let http = Http::new_with_token(&token);
	let (owners, bot_id) = match http.get_current_application_info().await {
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
async fn register(ctx: &Context, msg: &Message) -> CommandResult {
	Ok(())
}
