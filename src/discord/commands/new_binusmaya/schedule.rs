use chrono::{Duration, NaiveDate};
use futures::StreamExt;
use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::prelude::*,
    prelude::*,
};
use std::{ops::Add, str::FromStr};

use crate::{
    api::new_binusmaya_api::NewBinusmayaAPI,
    consts::{PRIMARY_COLOR, NEWBINUSMAYA_USER_DATA},
    discord::helper::Nav,
};

// async fn send_interactive_msg(ctx: &Context, msg: &Message, date: NaiveDate)

#[command]
#[num_args(1)]
#[description("Get the schedule of the given date")]
#[usage("[YYYY-MM-DD]")]
#[example("2022-01-05")]
async fn schedule(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let date = args.single::<String>().unwrap();
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
            let mut parsed_date = NaiveDate::parse_from_str(&date, "%Y-%-m-%-d").unwrap();
            let binusmaya_api = NewBinusmayaAPI {
                token: user_data
                    .lock()
                    .await
                    .get(msg.author.id.as_u64())
                    .unwrap()
                    .auth
                    .clone(),
            };
            let mut schedule = binusmaya_api.get_schedule(&parsed_date).await?;
            let mesg: Message;

            if let Some(class) = schedule {
                mesg = msg
                    .channel_id
                    .send_message(&ctx.http, |m| {
                        m.embed(|e| {
                            e.title(format!("Schedule for {}", date.clone()))
                                .description(format!(
                                    "**{} Session(s)**\n{}",
                                    class.schedule.len(),
                                    class
                                ))
                                .colour(PRIMARY_COLOR)
                        });
                        m.components(|c| c.add_action_row(Nav::action_row()))
                    })
                    .await?;
            } else {
                mesg = msg
                    .channel_id
                    .send_message(&ctx.http, |m| {
                        m.embed(|e| {
                            e.title(format!("Schedule for {}", date.clone()))
                                .colour(PRIMARY_COLOR)
                                .field("Holiday!", "No classes/sessions for today", true)
                        });
                        m.components(|c| c.add_action_row(Nav::action_row()))
                    })
                    .await?;
            }

            let mut cib = mesg
                .await_component_interactions(&ctx)
                .await;
            while let Some(mci) = cib.next().await {
                parsed_date = parsed_date.pred();
                let nav = Nav::from_str(&mci.data.custom_id).unwrap();
                match nav {
                    Nav::Previous => {
                        schedule = binusmaya_api.get_schedule(&parsed_date).await?;
                        if let Some(class) = schedule {
                            mci.create_interaction_response(&ctx, |r| {
                                r.kind(InteractionResponseType::UpdateMessage);
                                r.interaction_response_data(|m| {
                                    m.create_embed(|e| {
                                        e.title(format!("Schedule for {}", parsed_date.to_string()))
                                            .description(format!(
                                                "**{} Session(s)**\n{}",
                                                class.schedule.len(),
                                                class
                                            ))
                                            .colour(PRIMARY_COLOR)
                                    });
                                    m.components(|c| c.add_action_row(Nav::action_row()))
                                })
                            })
                            .await?;
                        } else {
                            mci.create_interaction_response(&ctx, |r| {
                                r.kind(InteractionResponseType::UpdateMessage);
                                r.interaction_response_data(|m| {
                                    m.create_embed(|e| {
                                        e.title(format!("Schedule for {}", parsed_date.to_string()))
                                            .colour(PRIMARY_COLOR)
                                            .field(
                                                "Holiday!",
                                                "No classes/sessions for today",
                                                true,
                                            )
                                    });
                                    m.components(|c| c.add_action_row(Nav::action_row()))
                                })
                            })
                            .await?;
                        }
                    }
                    Nav::Next => {
                        parsed_date = parsed_date.succ().succ();
                        schedule = binusmaya_api.get_schedule(&parsed_date).await?;
                        if let Some(class) = schedule {
                            mci.create_interaction_response(&ctx, |r| {
                                r.kind(InteractionResponseType::UpdateMessage);
                                r.interaction_response_data(|m| {
                                    m.create_embed(|e| {
                                        e.title(format!("Schedule for {}", parsed_date.to_string()))
                                            .description(format!(
                                                "**{} Session(s)**\n{}",
                                                class.schedule.len(),
                                                class
                                            ))
                                            .colour(PRIMARY_COLOR)
                                    });
                                    m.components(|c| c.add_action_row(Nav::action_row()))
                                })
                            })
                            .await?;
                        } else {
                            mci.create_interaction_response(&ctx, |r| {
                                r.kind(InteractionResponseType::UpdateMessage);
                                r.interaction_response_data(|m| {
                                    m.create_embed(|e| {
                                        e.title(format!("Schedule for {}", parsed_date.to_string()))
                                            .colour(PRIMARY_COLOR)
                                            .field(
                                                "Holiday!",
                                                "No classes/sessions for today",
                                                true,
                                            )
                                    });
                                    m.components(|c| c.add_action_row(Nav::action_row()))
                                })
                            })
                            .await?;
                        }
                    }
                }
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
