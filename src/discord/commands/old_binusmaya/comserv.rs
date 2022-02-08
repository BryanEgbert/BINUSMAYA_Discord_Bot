use serenity::framework::standard::CommandResult;
use serenity::framework::standard::macros::command;
use serenity::model::prelude::*;
use serenity::prelude::*;

use crate::{consts::{OLDBINUSMAYA_USER_DATA, PRIMARY_COLOR}, api::old_binusmaya_api::OldBinusmayaApi};

#[command]
async fn comserv(ctx: &Context, msg: &Message) -> CommandResult {
	
	Ok(())
}