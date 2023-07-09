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
#[description("Get ongoing classes")]
async fn ongoing(ctx: &Context, msg: &Message) -> CommandResult {
    // let user_data = NEWBINUSMAYA_USER_DATA.clone();
    let user_data_opt = consts::NEW_BINUSMAYA_REPO.get_by_id(msg.author.id.as_u64());

    if user_data_opt.as_ref().is_some_and(|user| user.is_ok()) {
        let data = user_data_opt.unwrap()?;

        let jwt_exp = data.last_registered.add(Duration::days(7));
        let now = chrono::offset::Local::now();

        if jwt_exp > now {
            let binusmaya_api = NewBinusmayaAPI {
                token: data.auth
            };

            let ongoing_sessions = binusmaya_api
                .get_ongoing_sessions()
                .await
                .expect("ongoing session error")
                .data;

            msg.channel_id
                .send_message(&ctx.http, |m| {
                    m.embed(|e| {
                        e.title("Ongoing Sessions")
                            .description(format!(
                                "**{} Ongoing Session(s)**\n{}",
                                ongoing_sessions.ongoing_classes.len(),
                                ongoing_sessions
                            ))
                            .colour(PRIMARY_COLOR)
                    })
                })
                .await?;
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
