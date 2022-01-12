use std::ops::Add;
use chrono::Duration;
use futures::{stream, StreamExt, future};
use serenity::framework::standard::{CommandResult, Args};
use serenity::framework::standard::macros::command;
use serenity::model::prelude::*;
use serenity::prelude::*;

use crate::{consts::{PRIMARY_COLOR, USER_DATA}, binusmaya::BinusmayaAPI};

#[command]
#[num_args(3)]
#[aliases("resource", "d")]
#[description("Get the link of the class and get the subtopics and resources of the session\n```Argument:\n[Subject name] - The name of the subject\n[Class component] - LAB/LEC/TUT\n[Session number] - Number of a session```")]
#[usage("[Subject name];[Class component];[Session number]")]
#[example("Linear;LEC;1")]
async fn details(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
	let course_name = args.single::<String>()?;
	let class_component = args.single::<String>()?;
	let session_number = args.single::<usize>()?;

	msg.react(&ctx, '👍').await?;

	let user_data = USER_DATA.lock().await;

	if user_data.contains_key(msg.author.id.as_u64()) {
		let jwt_exp = user_data.get(msg.author.id.as_u64()).unwrap().last_registered.add(Duration::weeks(52));
		let now = chrono::offset::Local::now();
		if jwt_exp > now {
			let binusmaya_api = BinusmayaAPI{token: user_data.get(msg.author.id.as_u64()).unwrap().auth.clone()};
	
			let class = stream::iter(binusmaya_api.get_classes().await?.classes)
				.filter(|c| future::ready(c.course_name.contains(&course_name) && c.ssr_component.eq(&class_component)))
				.next().await;
	
			if let Some(c) = class {
				let class_id = c.class_id;
				let class_details = binusmaya_api.get_class_details(class_id.clone()).await?;
		
				if class_details.sessions.len() < session_number {
					msg.channel_id.send_message(&ctx.http, |m| {
						m.embed(|e| e
							.colour(PRIMARY_COLOR)
							.field(format!("Session {} doesn't exists", session_number), format!("There's only {} Sessions", class_details.sessions.len()), false)
						)
					}).await.unwrap();
				} else {
					let session_id = &class_details.sessions[session_number - 1].id;
					let session_details = binusmaya_api.get_resource(session_id.to_string()).await.unwrap();
					msg.channel_id.send_message(&ctx.http, |m| {
						m.embed(|e| e
							.title(format!("{}\nSession {}", session_details.topic, session_details.session_number))
							.description(format!("**Class Zoom Link**\n{}\n\n**Subtopics**\n{}\n**Resources**\n{}", session_details.join_url.unwrap_or("No link".to_string()), session_details.course_sub_topic, session_details.resources))
							.colour(PRIMARY_COLOR)
							.url(format!("https://newbinusmaya.binus.ac.id/lms/course/{}/session/{}", class_id.clone(), session_id))
						)
					}).await?;
				}
			} else {
				msg.channel_id.send_message(&ctx.http, |m| {
					m.embed(|e| e
						.colour(PRIMARY_COLOR)
						.field(format!("subject named {} doesn't exists", course_name), "**Tips:** [subject name] and [class component] are case sensitive", false)
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