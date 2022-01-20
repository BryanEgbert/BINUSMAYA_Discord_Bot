use std::ops::Add;
use chrono::Duration;
use serenity::{
	framework::standard::{
		CommandResult, macros::command
	}, 
	model::prelude::*, 
	prelude::*
};
use scraper::{Html, Selector};

use crate::{consts::{PRIMARY_COLOR, USER_DATA}, binusmaya::{BinusmayaAPI, AnnouncementResponse, AnnouncementDetails}};

fn parse_html(content: &String) -> String {
	let mut parsed_content = String::new();

	let html = Html::parse_fragment(content);
	let selector = Selector::parse("p").unwrap();

	html.select(&selector).into_iter().for_each(|p| {
		let mut text = p.inner_html();
		if text.contains("<br>") {
			text = text.replace("<br>", "\n");
		}
		parsed_content.push_str(text.as_str());
		parsed_content.push('\n');
	});

	parsed_content
}

async fn send_announcement_details(ctx: &Context, msg: &Message, binusmaya_api: &BinusmayaAPI, announcement_list: &AnnouncementResponse) {
	let announcement_details: Option<AnnouncementDetails>;
	let title: String;
	let mut content = String::new();

	if let Some(answer) = &msg.author.await_reply(&ctx).timeout(Duration::seconds(30).to_std().unwrap()).await {
		let reply: usize = answer.content.parse().unwrap_or(0);
		if reply == 0 {
			return;
		}
		announcement_details = binusmaya_api.get_announcement_details(&announcement_list.announcements[reply - 1].id).await.unwrap_or(None);
		
		if let Some(announcement_details) = announcement_details {
			title = announcement_details.title;

			content.push_str(parse_html(&announcement_details.content).as_str());

			content.push_str(format!("**Attachment Link(s)\n{}**", announcement_details.attachment_links).as_str());
		} else {
			title = "Error".to_string();
			content = "Object reference not set to an instance of an object".to_string();
		}

		msg.channel_id.send_message(&ctx, |m|
			m.embed(|e| e
				.title(title)
				.description(content)
				.colour(PRIMARY_COLOR)
			)
		).await.unwrap();

	} else {
		return;
	}

}

#[command]
#[description("Get the announcements in new binusmaya")]
async fn announcement(ctx: &Context, msg: &Message) -> CommandResult {
	msg.react(&ctx, '👍').await?;

	let user_data = USER_DATA.clone();

	if user_data.lock().await.contains_key(msg.author.id.as_u64()) {
		let jwt_exp = user_data.lock().await.get(msg.author.id.as_u64()).unwrap().last_registered.add(Duration::weeks(52));
		let now = chrono::offset::Local::now();
		if jwt_exp > now {
			let binusmaya_api = BinusmayaAPI{token: user_data.lock().await.get(msg.author.id.as_u64()).unwrap().auth.clone()};
			// let mut page = 1;
			let announcement_list = binusmaya_api.get_announcement(1).await?;
	
			msg.channel_id.send_message(&ctx.http, |m| {
				m.embed(|e| e
					.title("Type the number to see the content")
					.description(&announcement_list)
					.colour(PRIMARY_COLOR)
					.footer(|f| f.text("Timeout in 30 seconds"))
				)
			}).await?;

			send_announcement_details(ctx, msg, &binusmaya_api, &announcement_list).await;
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