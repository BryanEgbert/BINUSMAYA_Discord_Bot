use serenity::framework::standard::CommandResult;
use serenity::framework::standard::macros::command;
use serenity::model::prelude::*;
use serenity::prelude::*;

use crate::consts::PRIMARY_COLOR;

#[command]
#[description = "send pong!"]
pub async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
	msg.channel_id.send_message(&ctx.http, |m| {
		m.embed(|e| e
			.colour(PRIMARY_COLOR)
			.title("reply to command")
			.field("test", "pong", false))
	}).await?;

	Ok(())
}