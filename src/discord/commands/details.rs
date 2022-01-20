use std::{ops::Add, str::FromStr};
use chrono::Duration;
use futures::{stream, StreamExt, future};
use serenity::framework::standard::{CommandResult, Args, macros::command};
use serenity::model::prelude::*;
use serenity::prelude::*;

use crate::discord::helper::Nav;
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
	let mut session_number = args.single::<usize>()?;

	msg.react(&ctx, '👍').await?;

	let user_data = USER_DATA.clone();

	if user_data.lock().await.contains_key(msg.author.id.as_u64()) {
		let jwt_exp = user_data.lock().await.get(msg.author.id.as_u64()).unwrap().last_registered.add(Duration::weeks(52));
		let now = chrono::offset::Local::now();
		if jwt_exp > now {
			let binusmaya_api = BinusmayaAPI{token: user_data.lock().await.get(msg.author.id.as_u64()).unwrap().auth.clone()};
	
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
					let mesg = msg.channel_id.send_message(&ctx.http, |m| {
						m.embed(|e| e
							.title(format!("{}\nSession {}", session_details.topic, session_details.session_number))
							.description(format!("**Class Zoom Link**\n{}\n\n**Subtopics**\n{}\n**Resources**\n{}", session_details.join_url.unwrap_or("No link".to_string()), session_details.course_sub_topic, session_details.resources))
							.colour(PRIMARY_COLOR)
							.url(format!("https://newbinusmaya.binus.ac.id/lms/course/{}/session/{}", class_id.clone(), session_id))
							.footer(|f| f.text(format!("session {}/{}", session_number, class_details.sessions.len())))
						);
						m.components(|c| c.add_action_row(Nav::action_row()));
						m
					}).await?;

					let mut cib = mesg.await_component_interactions(&ctx).timeout(Duration::seconds(30).to_std().unwrap()).await;
					while let Some(mci) = cib.next().await {
						let nav = Nav::from_str(&mci.data.custom_id).unwrap();
						match nav {
							Nav::Previous => {
								session_number = if session_number > 1 {
									session_number - 1
								} else {
									1
								};

								let session_id = &class_details.sessions[session_number - 1].id;
								let session_details = binusmaya_api.get_resource(session_id.to_string()).await.unwrap();
								
								mci.create_interaction_response(&ctx, |r| {
									r.kind(InteractionResponseType::UpdateMessage);
									r.interaction_response_data(|m| {
										m.create_embed(|e| e
											.title(format!("{}\nSession {}", session_details.topic, session_details.session_number))
											.description(format!("**Class Zoom Link**\n{}\n\n**Subtopics**\n{}\n**Resources**\n{}", session_details.join_url.unwrap_or("No link".to_string()), session_details.course_sub_topic, session_details.resources))
											.colour(PRIMARY_COLOR)
											.url(format!("https://newbinusmaya.binus.ac.id/lms/course/{}/session/{}", class_id.clone(), session_id))
											.footer(|f| f.text(format!("session {}/{}", session_number, class_details.sessions.len())))
										);
										m.components(|c| c.add_action_row(Nav::action_row()))
									})
								}).await?;
							},
							Nav::Next => {
								session_number = if session_number < class_details.sessions.len() {
									session_number + 1
								} else {
									class_details.sessions.len()
								};

								let session_id = &class_details.sessions[session_number - 1].id;
								let session_details = binusmaya_api.get_resource(session_id.to_string()).await.unwrap();

								mci.create_interaction_response(&ctx, |r| {
									r.kind(InteractionResponseType::UpdateMessage);
									r.interaction_response_data(|m| {
										m.create_embed(|e| e
											.title(format!("{}\nSession {}", session_details.topic, session_details.session_number))
											.description(format!("**Class Zoom Link**\n{}\n\n**Subtopics**\n{}\n**Resources**\n{}", session_details.join_url.unwrap_or("No link".to_string()), session_details.course_sub_topic, session_details.resources))
											.colour(PRIMARY_COLOR)
											.url(format!("https://newbinusmaya.binus.ac.id/lms/course/{}/session/{}", class_id.clone(), session_id))
											.footer(|f| f.text(format!("session {}/{}", session_number, class_details.sessions.len())))
										);
										m.components(|c| c.add_action_row(Nav::action_row()))
									})
								}).await?;
							}
						}
					}
				}
			} else {
				msg.channel_id.send_message(&ctx.http, |m| {
					m.embed(|e| e
						.colour(PRIMARY_COLOR)
						.field(format!("subject named {} doesn't exists", course_name), "**Tips:** `[subject name]` and `[class component]` are case sensitive", false)
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