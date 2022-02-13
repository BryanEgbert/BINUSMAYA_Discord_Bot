use std::fmt::Display;
use std::str::FromStr;
use futures::StreamExt;
use serenity::builder::{CreateActionRow, CreateButton};
use serenity::model::interactions::message_component::ButtonStyle;
use serenity::{framework::standard::CommandResult, builder::CreateSelectMenuOption};
use serenity::framework::standard::macros::command;
use serenity::model::prelude::*;
use serenity::prelude::*;

use crate::discord::helper::*;
use crate::{consts::{OLDBINUSMAYA_USER_DATA, PRIMARY_COLOR}, api::old_binusmaya_api::OldBinusmayaAPI, discord::helper::update_cookie};

enum AssignmentType {
	Individual,
	Group,
}

impl Display for AssignmentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Individual => write!(f, "Individual"),
			Self::Group => write!(f, "Group"),
		}
    }
}

impl FromStr for AssignmentType {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
			"Individual" => Ok(AssignmentType::Individual),
			"Group" => Ok(AssignmentType::Group),
			_ => Err(ParseError(s.to_string())),
		}
    }
}

impl AssignmentType {
	fn button(&self) -> CreateButton {
		let mut btn = CreateButton::default();
		btn.custom_id(self);
		btn.label(self);
		btn.style(ButtonStyle::Primary);

		btn
	}

	pub fn group_action_row() -> CreateActionRow {
		let mut ar = CreateActionRow::default();
		ar.add_button(Self::Group.button());
		
		ar
	}
	
	pub fn individual_action_row() -> CreateActionRow {
		let mut ar = CreateActionRow::default();
		ar.add_button(Self::Individual.button());
		
		ar
	}
}


async fn academic_period_menu_options(course_menu_list: &serde_json::Value) -> Vec<CreateSelectMenuOption> {
	let mut vec_opt: Vec<CreateSelectMenuOption> = Vec::new();
	let academic_period_list = course_menu_list[0][3].as_array().unwrap();

	academic_period_list.iter().enumerate().for_each(|(i, menu_list)| {
		let mut opt = CreateSelectMenuOption::default();
		opt.label(menu_list[1].as_str().unwrap());
		opt.value(i);

		vec_opt.push(opt);
	});

	vec_opt
}

async fn course_menu_options(course_menu_list: &serde_json::Value, academic_period_index: usize) -> Vec<CreateSelectMenuOption> {
	let mut vec_opt: Vec<CreateSelectMenuOption> = Vec::new();
	let course_list = course_menu_list[0][3][academic_period_index].as_array().unwrap();

	course_list.iter().enumerate().filter(|(i, _)| i > &1).for_each(|(i, c)| {
		let mut opt = CreateSelectMenuOption::default();
		opt.label(format!("{} - {}", c["CLASS_SECTION"].as_str().unwrap(), c["COURSE_TITLE_LONG"].as_str().unwrap()));
		opt.value(i);

		vec_opt.push(opt);
	});

	vec_opt
}


#[command]
#[description("Get list of assignments")]
#[aliases("as")]
async fn assignment(ctx: &Context, msg: &Message) -> CommandResult {
	let user_data = OLDBINUSMAYA_USER_DATA.clone();
	let user_data_content = user_data.lock().await;

	if user_data_content.contains_key(msg.author.id.as_u64()) {
		let cookie = user_data_content.get(msg.author.id.as_u64()).unwrap();
		let mut binusmaya_api = OldBinusmayaAPI { cookie: cookie.to_string() };
		let session_status = binusmaya_api.check_session().await?.session_status;

		if session_status == 0 {
			update_cookie(Some(*msg.author.id.as_u64())).await;
			binusmaya_api = OldBinusmayaAPI {cookie: cookie.to_string() };
		}

		let course_menu_list = binusmaya_api.get_course_menu_list().await.unwrap();

		let academic_period_select_menu = select_menu(academic_period_menu_options(&course_menu_list).await).await;

		let m = msg.channel_id.send_message(&ctx.http, |m| {
			m.content("Choose academic period");
			m.components(|c| c.add_action_row({
				let mut ar = CreateActionRow::default();
				ar.add_select_menu(academic_period_select_menu);

				ar
			}))
		}).await?;

		let mci = m.await_component_interaction(&ctx).await.unwrap();
		let academic_period_index: usize = mci.data.values.get(0).unwrap().parse().unwrap();

		let course_select_menu = select_menu(course_menu_options(&course_menu_list, academic_period_index).await).await;

		mci.create_interaction_response(&ctx, |r| {
			r.kind(InteractionResponseType::UpdateMessage);
			r.interaction_response_data(|d| {
				d.content("Choose course");
				d.components(|c| c.add_action_row ({
					let mut ar = CreateActionRow::default();
					ar.add_select_menu(course_select_menu);

					ar
				}))
			})
		}).await?;

		let mci = m.await_component_interaction(&ctx).await.unwrap();
		let course_index: usize = mci.data.values.get(0).unwrap().parse().unwrap();

		let chosen_course = &course_menu_list[0][3][academic_period_index][course_index];

		let individual_assignment = binusmaya_api.get_individual_assignments(chosen_course["CRSE_CODE"].as_str().unwrap(), chosen_course["CRSE_ID"].as_str().unwrap(), chosen_course["STRM"].as_str().unwrap(), chosen_course["SSR_COMPONENT"].as_str().unwrap(), chosen_course["CLASS_NBR"].as_str().unwrap()).await?;
		let group_assignment = binusmaya_api.get_group_assignments(chosen_course["CRSE_CODE"].as_str().unwrap(), chosen_course["CRSE_ID"].as_str().unwrap(), chosen_course["STRM"].as_str().unwrap(), chosen_course["SSR_COMPONENT"].as_str().unwrap(), chosen_course["CLASS_NBR"].as_str().unwrap()).await?;

		let url = format!("https://binusmaya.binus.ac.id/newStudent/#/class/assignment.{}/{}/{}/{}/{}", chosen_course["CRSE_CODE"].as_str().unwrap(), chosen_course["CRSE_ID"].as_str().unwrap(), chosen_course["STRM"].as_str().unwrap(), chosen_course["SSR_COMPONENT"].as_str().unwrap(), chosen_course["CLASS_NBR"].as_str().unwrap());

		mci.create_interaction_response(&ctx, |r| {
			r.kind(InteractionResponseType::UpdateMessage);
			r.interaction_response_data(|d| {
				d.content("");
				d.create_embed(|e| e
					.title("Individual Assignment(s)")
					.url(&url)
					.description(&individual_assignment)
					.colour(PRIMARY_COLOR)
				);
				d.components(|c| c.add_action_row(AssignmentType::group_action_row()))
			})
		}).await?;

		let mut cib = m.await_component_interactions(&ctx).await;
		while let Some(mci) = cib.next().await {
			let assignment_type = AssignmentType::from_str(&mci.data.custom_id).unwrap();
			
			match assignment_type {
    			AssignmentType::Individual => {

					mci.create_interaction_response(&ctx, |r| {
						r.kind(InteractionResponseType::UpdateMessage);
						r.interaction_response_data(|d| {
							d.content("");
							d.create_embed(|e| e
								.title("Individual Assignment(s)")
								.url(&url)
								.description(&individual_assignment)
								.colour(PRIMARY_COLOR)
							);
							d.components(|c| c.add_action_row(AssignmentType::group_action_row()))
						})
					}).await?;
				},
   				AssignmentType::Group => {

					mci.create_interaction_response(&ctx, |r| {
						r.kind(InteractionResponseType::UpdateMessage);
						r.interaction_response_data(|d| {
							d.create_embed(|e| e
								.title("Group Assignment(s)")
								.url(&url)
								.description(&group_assignment)
								.colour(PRIMARY_COLOR)
							);
							d.components(|c| c.add_action_row(AssignmentType::individual_action_row()))
						})
					}).await?;
				},
			}
		}
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