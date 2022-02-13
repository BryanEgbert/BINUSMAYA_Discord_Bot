use std::ops::Add;
use std::str::FromStr;

use chrono::Duration;
use futures::StreamExt;
use serenity::builder::{CreateSelectMenuOption, CreateActionRow};
use serenity::framework::standard::macros::command;
use serenity::framework::standard::CommandError;
use serenity::model::prelude::*;
use serenity::prelude::*;

use crate::api::new_binusmaya_api::NewBinusmayaAPI;
use crate::consts::{NEWBINUSMAYA_USER_DATA, PRIMARY_COLOR};
use crate::discord::helper::{Nav, select_menu};

async fn academic_period_menu_options(binusmaya_api: &NewBinusmayaAPI) -> Vec<CreateSelectMenuOption> {
    let mut vec_opt: Vec<CreateSelectMenuOption> = Vec::new();
    let academic_periods = binusmaya_api.get_academic_period().await.unwrap();
    for period in academic_periods {
        let mut opt = CreateSelectMenuOption::default();
        opt.label(format!("{}", period.academic_period_description));
        opt.value(period.academic_period);
        vec_opt.push(opt);
    }

    vec_opt
}

async fn class_component_menu_options(binusmaya_api: &NewBinusmayaAPI, academic_period: &str) -> Vec<CreateSelectMenuOption> {
	let mut vec_opt: Vec<CreateSelectMenuOption> = Vec::new();
	let class_components = binusmaya_api.get_component_list(academic_period).await.unwrap();

	for component in class_components {
		let mut opt = CreateSelectMenuOption::default();
		opt.label(&component);
		opt.value(&component);
		vec_opt.push(opt);
	}
	vec_opt
}

async fn course_menu_options(binusmaya_api: &NewBinusmayaAPI, academic_period: &str, class_component: &str) -> Vec<CreateSelectMenuOption> {
	let mut vec_opt: Vec<CreateSelectMenuOption> = Vec::new();
	let courses = binusmaya_api.get_component_courses(academic_period, class_component).await.unwrap();
 
	for course in courses {
		let mut opt = CreateSelectMenuOption::default();
		opt.label(&course.course_name);
		opt.value(&course.class_id);
		vec_opt.push(opt);
	}

	vec_opt
}

#[command]
#[description("Get session details")]
#[aliases("resource", "res")]
async fn session(ctx: &Context, msg: &Message) -> Result<(), CommandError> {
	 msg.react(&ctx, '👍').await?;

    let user_data = NEWBINUSMAYA_USER_DATA.clone();

    if user_data.lock().await.contains_key(msg.author.id.as_u64()) {
        let jwt_exp = user_data
            .lock()
            .await
            .get(msg.author.id.as_u64())
            .unwrap()
            .last_registered
            .add(Duration::weeks(52));
        let now = chrono::offset::Local::now();
        if jwt_exp > now {
			let binusmaya_api = NewBinusmayaAPI {
                token: user_data
                    .lock()
                    .await
                    .get(msg.author.id.as_u64())
                    .unwrap()
                    .auth
                    .clone(),
            };

			let academic_period_select_menu = select_menu(academic_period_menu_options(&binusmaya_api).await).await;
			let m = msg.channel_id.send_message(&ctx.http, |m| {
                m.content("Choose academic period");
                m.components(|c| c.add_action_row({
                    let mut ar = CreateActionRow::default();
                    ar.add_select_menu(academic_period_select_menu);

                    ar
                }))
            }).await?;

			let mci = m.await_component_interaction(&ctx).await.unwrap();
			let academic_period = mci.data.values.get(0).unwrap();

			let class_component_select_menu = select_menu(class_component_menu_options(&binusmaya_api, academic_period).await).await;
			
			mci.create_interaction_response(&ctx, |r| {
				r.kind(InteractionResponseType::UpdateMessage);
				r.interaction_response_data(|d| {
					d.content("Choose class component");
					d.components(|c| c.add_action_row({
						let mut ar = CreateActionRow::default();
						ar.add_select_menu(class_component_select_menu);

						ar
					}))
				})
			}).await?;

			let mci = m.await_component_interaction(&ctx).await.unwrap();
			let class_component = mci.data.values.get(0).unwrap();

			let course_select_menu = select_menu(course_menu_options(&binusmaya_api, academic_period, class_component).await).await;

			mci.create_interaction_response(&ctx, |r| {
				r.kind(InteractionResponseType::UpdateMessage);
				r.interaction_response_data(|d| {
					d.content("Choose course");
					d.components(|c| c.add_action_row({
						let mut ar = CreateActionRow::default();
						ar.add_select_menu(course_select_menu);

						ar
					}))
				})
			}).await?;

			let mci = m.await_component_interaction(&ctx).await.unwrap();
			let class_id = mci.data.values.get(0).unwrap();
			let class_details = binusmaya_api.get_class_details(class_id.to_string()).await?;

			mci.create_interaction_response(&ctx, |r| {
				r.kind(InteractionResponseType::ChannelMessageWithSource);
				r.interaction_response_data(|d| {
					d.create_embed(|e| e
						.field("Choose Session Number", format!("Choose session number from 1 - {}", class_details.sessions.len()), false)
						.footer(|f| f.text("Timeout in 30 seconds, type cancel to cancel operation"))
						.colour(PRIMARY_COLOR)
					)
				})
			}).await?;

			if let Some(reply) = &msg.author.await_reply(&ctx).timeout(Duration::seconds(30).to_std().unwrap()).await {
				if reply.content.eq("cancel") {
					msg.react(&ctx, '👍').await?;
					return Ok(());
				}

				let mut session_num: usize = reply.content.parse().unwrap_or(1);

				if session_num > class_details.sessions.len() {
					session_num = class_details.sessions.len();
				} else if session_num < 1 {
					session_num = 1;
				}

				let session_id = &class_details.sessions[session_num - 1].id;
				let session_details = binusmaya_api.get_resource(session_id.to_string()).await?;

				let mesg = msg.channel_id.send_message(&ctx.http, |m| {
					m.embed(|e| e
						.title(format!("{}\nSession {}", session_details.topic, session_details.session_number))
						.description(format!("**Class Zoom Link**\n{}\n\n**Subtopics**\n{}\n**Resources**\n{}", session_details.join_url.unwrap_or("No link".to_string()), session_details.course_sub_topic, session_details.resources))
						.colour(PRIMARY_COLOR)
						.url(format!("https://newbinusmaya.binus.ac.id/lms/course/{}/session/{}", class_id.clone(), session_id))
						.footer(|f| f.text(format!("session {}/{}", session_num, class_details.sessions.len())))
					);
					m.components(|c| c.add_action_row(Nav::action_row()));
					m
				}).await?;

				let mut cib = mesg.await_component_interactions(&ctx).await;
				while let Some(mci) = cib.next().await {
					let nav = Nav::from_str(&mci.data.custom_id).unwrap();
					match nav {
						Nav::Previous => {
							session_num = if session_num > 1 {
								session_num - 1
							} else {
								1
							};

							let session_id = &class_details.sessions[session_num - 1].id;
							let session_details = binusmaya_api
								.get_resource(session_id.to_string())
								.await
								.unwrap();

							mci.create_interaction_response(&ctx, |r| {
								r.kind(InteractionResponseType::UpdateMessage);
								r.interaction_response_data(|m| {
									m.create_embed(|e| e
										.title(format!("{}\nSession {}", session_details.topic, session_details.session_number))
										.description(format!("**Class Zoom Link**\n{}\n\n**Subtopics**\n{}\n**Resources**\n{}", session_details.join_url.unwrap_or("No link".to_string()), session_details.course_sub_topic, session_details.resources))
										.colour(PRIMARY_COLOR)
										.url(format!("https://newbinusmaya.binus.ac.id/lms/course/{}/session/{}", class_id.clone(), session_id))
										.footer(|f| f.text(format!("session {}/{}", session_num, class_details.sessions.len())))
									);
									m.components(|c| c.add_action_row(Nav::action_row()))
								})
							}).await?;
						}
						Nav::Next => {
							session_num = if session_num < class_details.sessions.len() {
								session_num + 1
							} else {
								class_details.sessions.len()
							};

							let session_id = &class_details.sessions[session_num - 1].id;
							let session_details = binusmaya_api
								.get_resource(session_id.to_string())
								.await
								.unwrap();

							mci.create_interaction_response(&ctx, |r| {
								r.kind(InteractionResponseType::UpdateMessage);
								r.interaction_response_data(|m| {
									m.create_embed(|e| e
										.title(format!("{}\nSession {}", session_details.topic, session_details.session_number))
										.description(format!("**Class Zoom Link**\n{}\n\n**Subtopics**\n{}\n**Resources**\n{}", session_details.join_url.unwrap_or("No link".to_string()), session_details.course_sub_topic, session_details.resources))
										.colour(PRIMARY_COLOR)
										.url(format!("https://newbinusmaya.binus.ac.id/lms/course/{}/session/{}", class_id.clone(), session_id))
										.footer(|f| f.text(format!("session {}/{}", session_num, class_details.sessions.len())))
									);
									m.components(|c| c.add_action_row(Nav::action_row()))
								})
							}).await?;
						}
					}
				}
			} else {
				return Ok(());
			}
		} else {
			msg.channel_id
			.send_message(&ctx.http, |m| {
				m.embed(|e| {
					e.colour(PRIMARY_COLOR).field(
						"Your bearer token has expired",
						"please re-register using `=add` command",
						false,
					)
				})
			})
			.await?;
		}
	} else {
        msg.channel_id
            .send_message(&ctx.http, |m| {
                m.embed(|e| {
                    e.colour(PRIMARY_COLOR).field(
                        "You're not registered",
                        "please register first using `=register` command",
                        false,
                    )
                })
            })
            .await?;
    }
	
	Ok(())
}