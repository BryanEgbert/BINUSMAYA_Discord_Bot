use serenity::framework::standard::CommandResult;
use serenity::framework::standard::macros::command;
use serenity::model::prelude::*;
use serenity::prelude::*;

use crate::{consts::{OLDBINUSMAYA_USER_DATA, PRIMARY_COLOR}, api::old_binusmaya_api::OldBinusmayaApi, discord::helper::update_cookie};

#[command]
async fn comserv(ctx: &Context, msg: &Message) -> CommandResult {
	msg.react(&ctx, '👍').await?;
	
	let user_data = OLDBINUSMAYA_USER_DATA.clone();
	let user_data_content = user_data.lock().await;
	
	if user_data_content.contains_key(msg.author.id.as_u64()) {
		let cookie = user_data_content.get(msg.author.id.as_u64()).unwrap();
		let mut binusmaya_api = OldBinusmayaApi { cookie: cookie.to_string() };
		let session_status = binusmaya_api.check_session().await?.session_status;

		if session_status == 0 {
			update_cookie(Some(*msg.author.id.as_u64())).await;
			binusmaya_api = OldBinusmayaApi { cookie: cookie.to_string() };
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