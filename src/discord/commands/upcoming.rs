use std::ops::Add;

use chrono::Duration;
use serenity::{
    framework::standard::{macros::command, CommandResult},
    model::prelude::*,
    prelude::*,
};

use crate::{
    binusmaya::BinusmayaAPI,
    consts::{PRIMARY_COLOR, NEWBINUSMAYA_USER_DATA},
};

#[command]
#[description("Get upcoming sessions")]
async fn upcoming(ctx: &Context, msg: &Message) -> CommandResult {
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
            let binusmaya_api = BinusmayaAPI {
                token: user_data
                    .lock()
                    .await
                    .get(msg.author.id.as_u64())
                    .unwrap()
                    .auth
                    .clone(),
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
