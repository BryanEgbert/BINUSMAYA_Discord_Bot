use std::ops::Add;
use chrono::Duration;
use serenity::framework::standard::{CommandResult, Args};
use serenity::framework::standard::macros::command;
use serenity::model::prelude::*;
use serenity::prelude::*;

use crate::{consts::{PRIMARY_COLOR, USER_DATA}, binusmaya::BinusmayaAPI};

#[command]
#[num_args(1)]
#[description("Get the schedule of the given date")]
#[usage("[YYYY-MM-DD]")]
#[example("2022-01-05")]
async fn schedule(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
	let date = args.single::<String>().unwrap();
	let user_data = USER_DATA.lock().await;

	msg.react(&ctx, '👍').await?;

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