use std::ops::Add;

use chrono::Duration;
use serenity::{
	framework::standard::{
		CommandResult, macros::command
	}, 
	model::prelude::*, 
	prelude::*
};

use crate::{consts::{PRIMARY_COLOR, USER_DATA}, binusmaya::BinusmayaAPI};

#[command]
#[description("Get ongoing classes")]
async fn ongoing(ctx: &Context, msg: &Message) -> CommandResult {
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
			let ongoing_sessions = binusmaya_api.get_ongoing_sessions().await.expect("ongoing session error").data;
			bot_msg.edit(&ctx.http, |m|
				m.embed(|e| e 
					.title("Ongoing Sessions")
					.description(format!("**{} Ongoing Session(s)**\n{}", ongoing_sessions.ongoing_classes.len(), ongoing_sessions))
					.colour(PRIMARY_COLOR)
				)).await?;
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