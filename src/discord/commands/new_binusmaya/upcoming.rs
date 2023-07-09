use std::ops::Add;

use chrono::Duration;
use serenity::{
    framework::standard::{macros::command, CommandResult},
    model::prelude::*,
    prelude::*,
};

use crate::{
    api::new_binusmaya_api::NewBinusmayaAPI,
    consts::{PRIMARY_COLOR, self},
};

#[command]
#[description("Get upcoming sessions")]
async fn upcoming(ctx: &Context, msg: &Message) -> CommandResult {
    let user_data_opt = consts::NEW_BINUSMAYA_REPO.get_by_id(msg.author.id.as_u64());

    if user_data_opt.as_ref().is_some_and(|user| user.is_ok()) {
        let data = user_data_opt.unwrap()?;

        let jwt_exp = data.last_registered.add(Duration::days(7));
        let now = chrono::offset::Local::now();

        if jwt_exp > now {
            let binusmaya_api = NewBinusmayaAPI {
                token: data.auth
            };

            let upcoming_session = binusmaya_api.get_upcoming_sessions().await.unwrap_or(None);

            if let Some(session) = upcoming_session {
                msg.channel_id
                    .send_message(&ctx.http, |m| {
                        m.embed(|e| {
                            e.title("Upcoming Session")
                                .description(session)
                                .colour(PRIMARY_COLOR)
                        })
                    })
                    .await?;
            } else {
                msg.channel_id
                    .send_message(&ctx.http, |m| {
                        m.embed(|e| {
                            e.title("Upcoming Session")
                                .description(format!("{}", "No upcoming session"))
                                .colour(PRIMARY_COLOR)
                        })
                    })
                    .await?;
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
