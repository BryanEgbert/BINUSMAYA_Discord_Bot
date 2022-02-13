use serenity::{
    framework::standard::{macros::command, CommandResult},
    model::prelude::*,
    prelude::*,
};

use crate::consts::PRIMARY_COLOR;

#[command]
async fn about(ctx: &Context, msg: &Message) -> CommandResult {
    msg.react(&ctx, '👍').await?;

    msg.channel_id.send_message(&ctx, |m| {
		m.embed(|e| e
			.colour(PRIMARY_COLOR)
			.field("Made By:", "Bryan Egbert `PlayerPlay#9549`", false)
			.field("Version:", "`v2.0.0`", true)
			.field("Releases", "[Click here](https://github.com/BryanEgbert/BINUSMAYA_Discord_Bot/releases)", true)
			.field("Bot General Info", "[Click here](https://github.com/BryanEgbert/BINUSMAYA_Discord_Bot)", false)
			.footer(|f| f.text("This bot is open source. Any feedback or feature request is welcome"))
		)
	}).await?;
    Ok(())
}
