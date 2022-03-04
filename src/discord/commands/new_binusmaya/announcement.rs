use chrono::Duration;
use pcre2::bytes::RegexBuilder;
use serenity::{
    framework::standard::{macros::command, CommandResult},
    model::{prelude::*, interactions::message_component::ButtonStyle},
    prelude::*, builder::CreateButton,
};
use std::ops::Add;

use crate::{
    api::new_binusmaya_api::{AnnouncementDetails, AnnouncementResponse, NewBinusmayaAPI},
    consts::{PRIMARY_COLOR, NEWBINUSMAYA_USER_DATA},
};

fn parse_html(mut content: String) -> String {
    let mut parsed_content = String::new();
    let content_clone = content.clone();

    let reg = RegexBuilder::new().build(r"(?P<open_tag><.*?>)").unwrap();
    
    for tags in reg.captures_iter(&content_clone.as_bytes()) {
        let caps = tags.unwrap();
        let open_tag = std::str::from_utf8(&caps["open_tag"]).unwrap();

        if open_tag.contains("<li") {
           content = content.replace(open_tag,"- ");
        } else if open_tag.contains("<strong") || open_tag.eq("</strong>"){
           content = content.replace(open_tag, "**");
        } else if open_tag.contains("<em") || open_tag.eq("</em>") {
           content = content.replace(open_tag, "*");
        } else if open_tag.contains("<span") || open_tag.eq("</span>") {
           content = content.replace(open_tag, "__");
        } else if open_tag.eq("</p>") || open_tag.eq("<br />"){
           content = content.replace(open_tag, "\n");
        } else {
           content = content.replace(open_tag, "");
        }

       content = content.replace("&nbsp;", " ").replace("&ndash;", "-").replace("&amp;", "&").replace("&ge;", "≥").replace("&zwj;", "").replace("&bull;", "- ");     
    }

    parsed_content.push_str(&content); 
    
    parsed_content
}

async fn send_announcement_details(
    ctx: &Context,
    msg: &Message,
    binusmaya_api: &NewBinusmayaAPI,
    announcement_list: &AnnouncementResponse,
) {
    let announcement_details: Option<AnnouncementDetails>;

    if let Some(answer) = &msg
        .author
        .await_reply(&ctx)
        .timeout(Duration::seconds(30).to_std().unwrap())
        .await
    {
        let reply: usize = answer.content.parse().unwrap_or(0);
        if reply == 0 {
            return;
        }
        announcement_details = binusmaya_api
            .get_announcement_details(&announcement_list.announcements[reply - 1].id)
            .await
            .unwrap_or(None);

        if let Some(details) = announcement_details {
            msg.channel_id
                .send_message(&ctx, |m| {
                    m.embed(|e| e.title(details.title).description(parse_html(details.content)).colour(PRIMARY_COLOR));
                    m.components(|f| {
                        f.create_action_row(|ar| {
                            if details.attachment_links.is_empty() {
                                return ar;
                            }
    
                            details.attachment_links.iter().for_each(|link| {
                                let mut btn = CreateButton::default();
                                btn.style(ButtonStyle::Link);
                                btn.url(link.clone().unwrap());
                                btn.label("Attachment Link");

                                ar.add_button(btn);
                            });
    
                            ar
                        })
                    })
                })
                .await
                .unwrap();
        } else {
            msg.channel_id.send_message(&ctx, |m| {
                m.embed(|e| e.title("Error").description("Object reference not set to an instance of an object").colour(PRIMARY_COLOR))
            }).await.unwrap();
        }

    } else {
        return;
    }
}

#[command]
#[description("Get the announcements in new binusmaya")]
async fn announcement(ctx: &Context, msg: &Message) -> CommandResult {
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
            // let mut page = 1;
            let announcement_list = binusmaya_api.get_announcement(1).await?;

            msg.channel_id
                .send_message(&ctx.http, |m| {
                    m.embed(|e| {
                        e.title("Type the number to see the content")
                            .description(&announcement_list)
                            .colour(PRIMARY_COLOR)
                            .footer(|f| f.text("Timeout in 30 seconds"))
                    })
                })
                .await?;

            send_announcement_details(ctx, msg, &binusmaya_api, &announcement_list).await;
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
