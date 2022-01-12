use std::ops::Add;
use chrono::Duration;
use serenity::framework::standard::CommandResult;
use serenity::framework::standard::macros::command;
use serenity::model::prelude::*;
use serenity::prelude::*;

use crate::{consts::{PRIMARY_COLOR, USER_DATA}, binusmaya::BinusmayaAPI};

#[command]
#[aliases("c")]
#[description("Get the list of classes in your major")]
pub async fn classes(ctx: &Context, msg: &Message) -> CommandResult {
	
	let user_data = USER_DATA.lock().await;
	
	if user_data.contains_key(msg.author.id.as_u64()) {
		let jwt_exp = user_data.get(msg.author.id.as_u64()).unwrap().last_registered.add(Duration::weeks(52));
		let now = chrono::offset::Local::now();
		if jwt_exp > now {
			let mut bot_msg = msg.channel_id.send_message(&ctx.http, |m| {
				m.embed(|e| e
					.field("Loading...", "Fetching data", false)
					.colour(PRIMARY_COLOR)
				)
			}).await?;
			let binusmaya_api = BinusmayaAPI{token: user_data.get(msg.author.id.as_u64()).unwrap().auth.clone()};
			let classes = binusmaya_api.get_classes().await?;
	
			bot_msg.edit(&ctx.http, |m| {
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