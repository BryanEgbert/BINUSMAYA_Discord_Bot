use chrono::Duration;
use pcre2::bytes::RegexBuilder;
use serenity::{
    framework::standard::{macros::command, CommandResult},
    model::prelude::*,
    prelude::*,
};
use std::ops::Add;

use crate::{
    binusmaya::{AnnouncementDetails, AnnouncementResponse, BinusmayaAPI},
    consts::{PRIMARY_COLOR, USER_DATA},
};

fn parse_html(content: &str) -> String {
    let mut parsed_content = String::new();
    let content2 = String::from(content);
    let reg = RegexBuilder::new().build(r"(?P<open_tag><.*?>)(?P<inner_html>.*?)(?P<close_tag></.*?>)").unwrap();

    for tags in reg.captures_iter(&content2.as_bytes()) {
        let caps = tags.unwrap();
        let open_tag = std::str::from_utf8(&caps["open_tag"]).unwrap();
        let content = std::str::from_utf8(&caps["inner_html"]).unwrap();

        if open_tag.contains("<li") {
            parsed_content.push_str("- ");
        }

        if content.eq("&nbsp;") {
            parsed_content.push_str(content.replace("&nbsp;", "\n").as_str());
        } else if content.contains("<strong>") && content.contains("<span style=\"text-decoration: underline;\">") {
            parsed_content.push_str(content.replace("<strong>", "").replace("<span style=\"text-decoration: underline;\">", "__").replace("&nbsp;", " ").as_str());
            parsed_content.push_str("__");
        } else if content.contains("<em>") {
            parsed_content.push_str(content.replace("<em>", "*").as_str());
            parsed_content.push('*');
        } else if content.contains("<a") {
            let mut content = String::from(content);

            let start_tag = content.find("<a").unwrap();
            let end_tag = content.rfind(">").unwrap();

            content.replace_range(start_tag..end_tag+1, "");

            parsed_content.push_str(content.replace("<span style=\"text-decoration: underline;\">", "__").replace("&nbsp;", " ").as_str());
            parsed_content.push_str("__");
        } else if content.contains("<span style=\"text-decoration: underline;\">") {
            parsed_content.push_str(content.replace("<span style=\"text-decoration: underline;\">", "__").replace("&nbsp;", " ").as_str());
            parsed_content.push_str("__");
        } else if content.contains("<strong>") {
            parsed_content.push_str(content.replace("&nbsp;", " ").replace("<strong>", "**").replace("&ge;", "≥").as_str());
            parsed_content.push_str("**");
        } else {
             parsed_content.push_str(content.replace("&nbsp;", " ").replace("&ge;", "≥").as_str());
        }

        parsed_content.push('\n');        
        println!("open: {:?}\tcontent:{:?}\tclose: {:?}", std::str::from_utf8(&caps["open_tag"]).unwrap(), std::str::from_utf8(&caps["inner_html"]).unwrap(), std::str::from_utf8(&caps["close_tag"]).unwrap());
    }
    // let v: Vec<&str> = content.split_inclusive('>').collect();
    // println!("{:?}", &v);

    // v.clone().iter().enumerate().for_each(|(i, string)| {
    //     let owned_string = string.to_owned();
    //     if owned_string.contains("<p>") || owned_string.contains("</p>") || owned_string.contains("<div>") || owned_string.contains("</div>") || owned_string.contains("<ul>") || owned_string.contains("</ul>") || owned_string.contains("<li>") {
    //         v[i] = v[i].replace("<p>", "").as_str();
    //     }

    // });

    // parsed_content.push_str(&content
    //     .replace("<p>", "")
    //     .replace("</p>", "")
    //     .replace("<div>", "")
    //     .replace("</div>", "")
    //     .replace("<br>", "\n")
    //     .replace("<br />", "\n")
    //     .replace("<em>", "*")
    //     .replace("</em>", "*")
    //     .replace("<strong>", "**")
    //     .replace("</strong>", "**")
    //     .replace("<li>", "  - ")
    //     .replace("</li>", "")
    //     .replace("<ul>", "")
    //     .replace("</ul>", "")
    //     .replace("&amp;", "&")
    //     .replace("&nbsp;", " ")
    //     .replace("&ndash;", "-")
    //     .replace("&ge;", "≥")
    // );


    parsed_content
}

async fn send_announcement_details(
    ctx: &Context,
    msg: &Message,
    binusmaya_api: &BinusmayaAPI,
    announcement_list: &AnnouncementResponse,
) {
    let announcement_details: Option<AnnouncementDetails>;
    let title: String;
    let mut content = String::new();

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

        if let Some(announcement_details) = announcement_details {
            title = announcement_details.title;

            content.push_str(parse_html(&announcement_details.content).as_str());

            content.push_str(
                format!(
                    "**Attachment Link(s)\n{}**",
                    announcement_details.attachment_links
                )
                .as_str(),
            );
        } else {
            title = "Error".to_string();
            content = "Object reference not set to an instance of an object".to_string();
        }

        msg.channel_id
            .send_message(&ctx, |m| {
                m.embed(|e| e.title(title).description(content).colour(PRIMARY_COLOR))
            })
            .await
            .unwrap();
    } else {
        return;
    }
}

#[command]
#[description("Get the announcements in new binusmaya")]
async fn announcement(ctx: &Context, msg: &Message) -> CommandResult {
    msg.react(&ctx, '👍').await?;

    let user_data = USER_DATA.clone();

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
