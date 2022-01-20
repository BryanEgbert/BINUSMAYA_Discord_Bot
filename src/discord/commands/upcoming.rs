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
#[description("Get upcoming sessions")]
async fn upcoming(ctx: &Context, msg: &Message) -> CommandResult {
	msg.react(&ctx, '👍').await?;

	let user_data = USER_DATA.clone();
	
	if user_data.lock().await.contains_key(msg.author.id.as_u64()) {
		let jwt_exp = user_data.lock().await.get(msg.author.id.as_u64()).unwrap().last_registered.add(Duration::weeks(52));
		let now = chrono::offset::Local::now();

		if jwt_exp > now {
			let binusmaya_api = BinusmayaAPI{token: user_data.lock().await.get(msg.author.id.as_u64()).unwrap().auth.clone()};
			let upcoming_session = binusmaya_api.get_upcoming_sessions().await?;

			msg.channel_id.send_message(&ctx.http, |m|
				m.embed(|e| e 
						.title("Upcoming Session")
						.description(format!("{}", upcoming_session))
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