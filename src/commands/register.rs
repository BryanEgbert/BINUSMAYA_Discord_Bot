use serenity::framework::standard::CommandResult;
use serenity::framework::standard::macros::command;
use serenity::model::prelude::*;
use serenity::prelude::*;

use crate::consts::PRIMARY_COLOR;


#[command]
#[description("Receive a DM to register your binus account for additional features")]
#[only_in("guild")]
pub async fn register(ctx: &Context, msg: &Message) -> CommandResult {
	msg.react(&ctx, '👍').await?;
	
	msg.author.dm(&ctx, |m| {
		m.embed(|e| e
			.colour(PRIMARY_COLOR)
			.field("Register", "Please enter your BINUS email and password, e.g. `=add [email];[password]`", false))
	}).await?;

	Ok(())
}