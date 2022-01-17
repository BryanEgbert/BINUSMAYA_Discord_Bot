use serenity::client::bridge::gateway::ShardId;
use serenity::framework::standard::CommandResult;
use serenity::framework::standard::macros::command;
use serenity::model::prelude::*;
use serenity::prelude::*;

use crate::consts::PRIMARY_COLOR;
use crate::discord::discord::ShardManagerContainer;

#[command]
#[description = "send pong!"]
pub async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
	msg.react(&ctx, '👍').await?;
	
	let latency = {
		let data = ctx.data.read().await;
		let shard_manager = data.get::<ShardManagerContainer>().unwrap();

		let manager = shard_manager.lock().await;
		let runners = manager.runners.lock().await;

		let runner = runners.get(&ShardId(ctx.shard_id)).unwrap();

		if let Some(duration) = runner.latency {
			format!("{:.2}ms", duration.as_millis())
		} else {
			"?ms".to_string()
		}
	};

	msg.channel_id.send_message(&ctx.http, |m| {
		m.embed(|e| e
			.colour(PRIMARY_COLOR)
			.field("Shard latency", format!("The shard latency is: **{}**", latency), false))
	}).await?;

	Ok(())
}