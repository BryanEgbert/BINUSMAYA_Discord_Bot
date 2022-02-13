use chrono::Duration;
use serenity::framework::standard::macros::command;
use serenity::framework::standard::CommandResult;
use serenity::model::prelude::*;
use serenity::prelude::*;
use std::ops::Add;

use crate::{
    api::new_binusmaya_api::NewBinusmayaAPI,
    consts::{PRIMARY_COLOR, NEWBINUSMAYA_USER_DATA},
};

#[command]
#[aliases("c")]
#[description("Get the list of active classes in your major")]
pub async fn classes(ctx: &Context, msg: &Message) -> CommandResult {
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
            let classes = binusmaya_api.get_classes().await?;

            msg.channel_id
                .send_message(&ctx.http, |m| {
                    m.embed(|e| {
                        e.title("Class List")
                            .description(classes)
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
