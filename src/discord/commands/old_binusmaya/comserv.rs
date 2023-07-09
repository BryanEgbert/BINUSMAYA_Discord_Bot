use serenity::framework::standard::CommandResult;
use serenity::framework::standard::macros::command;
use serenity::model::prelude::*;
use serenity::prelude::*;

use crate::{consts::{PRIMARY_COLOR, self}, api::old_binusmaya_api::OldBinusmayaAPI, discord::helper::update_cookie};

#[command]
async fn comserv(ctx: &Context, msg: &Message) -> CommandResult {
	let user_data = consts::OLD_BINUSMAYA_REPO.get_by_id(msg.author.id.as_u64());
	
	if user_data.as_ref().is_some_and(|user| user.is_ok()) {
		let binusmaya_api = OldBinusmayaAPI { cookie: user_data.unwrap()?.cookie };
		let session_status = binusmaya_api.check_session().await?.session_status;

		if session_status == 0 {
			update_cookie(
				&consts::OLD_BINUSMAYA_REPO, 
				msg.author.id.as_u64(), 
			).await;
		}

		let comserv = binusmaya_api.get_comnunity_service().await?;

		msg.channel_id.send_message(&ctx.http, |m| {
			m.embed(|e| e
				.colour(PRIMARY_COLOR)
				.field("Community Service", comserv, true))
		}).await?;
	} else {
		msg.channel_id.send_message(&ctx.http, |m| {
			m.embed(|e| e
				.colour(PRIMARY_COLOR)
				.field("You're Not Registered", "You haven't registered yet, use `=register` command to register your account", false)
			)
		}).await?;
	}
	
	Ok(())
}