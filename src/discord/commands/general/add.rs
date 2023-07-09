use chrono::{Duration, Local};

use magic_crypt::MagicCryptTrait;
use serenity::{
    framework::standard::{macros::command, CommandResult},
    model::prelude::*,
    prelude::*, builder::{CreateSelectMenuOption, CreateSelectMenu, CreateActionRow},
};
use std::{str::FromStr, fmt};
use std::ops::Add;

use thirtyfour::{error::WebDriverError, DesiredCapabilities, Proxy, WebDriver, CapabilitiesHelper};

use crate::{
    consts::{NEW_BINUSMAYA, PRIMARY_COLOR, OLD_BINUSMAYA, CHROME_SERVER_URL, MAGIC_CRYPT, self},
    discord::{helper::ParseError},
    api::{old_binusmaya_api::{OldBinusmayaAPI}},
    third_party::{BrowserMobProxy, Selenium, Status}, models, repository,
};

#[derive(Debug)]
enum CookieOutput<T,E> {
	Out(T, E)
}

#[derive(Clone, Debug)]
enum Binusmaya {
    NewBinusmaya,
    OldBinusmaya
}

impl fmt::Display for Binusmaya {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NewBinusmaya => write!(f, "New Binusmaya"),
            Self::OldBinusmaya => write!(f, "Old Binusmaya"),
        }
    }
}

impl FromStr for Binusmaya {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Old Binusmaya" => Ok(Binusmaya::OldBinusmaya),
            "New Binusmaya" => Ok(Binusmaya::NewBinusmaya),
            _ => Err(ParseError(s.to_string())),
        }
    }
}

impl Binusmaya {
    fn menu_option(&self) -> CreateSelectMenuOption {
        let mut opt = CreateSelectMenuOption::default();
        opt.label(self);
        opt.value(self);

        opt
    }

    fn select_menu() -> CreateSelectMenu {
        let mut menu = CreateSelectMenu::default();
        menu.custom_id("binusmaya_select");
        menu.placeholder("Choose version of Binusmaya");
        menu.options(|f| {
            f.add_option(Self::NewBinusmaya.menu_option());
            f.add_option(Self::OldBinusmaya.menu_option())
        });

        menu
    }

    fn action_row() -> CreateActionRow {
        let mut ar = CreateActionRow::default();
        ar.add_select_menu(Self::select_menu());

        ar
    }
}



async fn launch_selenium(
    user_credential: &models::user::UserCredential,
    proxy: &BrowserMobProxy,
    link: String,
) -> Result<CookieOutput<Status<String>, Option<String>>, WebDriverError> {
    proxy.create_proxy().await.unwrap();

    let proxy_port = proxy.get_proxy().await.unwrap();
    let index = proxy_port.proxyList.len() - 1;

    let mut caps = DesiredCapabilities::chrome();
    caps.set_proxy(Proxy::Manual {
        ftp_proxy: None,
        http_proxy: Some(format!(
            "http://{}:{}",
            proxy.host, proxy_port.proxyList[index].port
        )),
        ssl_proxy: Some(format!(
            "http://{}:{}",
            proxy.host, proxy_port.proxyList[index].port
        )),
        socks_proxy: None,
        socks_version: None,
        socks_username: None,
        socks_password: None,
        no_proxy: None,
    })?;

    
    // caps.accept_ssl_certs(true)?;
    // caps.set_binary(CHROME_BINARY.lock().await.as_str())?;
    caps.add_chrome_arg("--proxy-server=http://192.168.100.25:20000")?;
    caps.add_chrome_arg("--ignore-certificate-errors")?;
    caps.add("acceptInsecureCerts", true)?;
    // caps.set_headless()?;

    let selenium = Selenium::init(
        WebDriver::new(CHROME_SERVER_URL, caps).await?,
        user_credential.email.clone(),
        user_credential.password.clone(),
    );

    selenium.setup().await?;

    BrowserMobProxy::new_har(&proxy).await.unwrap();
    let is_valid = selenium.run(&link.to_string()).await.unwrap_or(
		Status::ERROR("Error in registering, please try again. If the problem still persist, please contact `PlayerPlay#9549` or open a new issue [here](https://github.com/BryanEgbert/BINUSMAYA_Discord_Bot/issues)".to_string())
	);

    if let Status::VALID(_) = is_valid {
        let mut cookie: Option<String> = None;
        if link.eq(OLD_BINUSMAYA) {
            let har = BrowserMobProxy::get_har(&proxy).await.unwrap();

            for entry in har["log"]["entries"].as_array().unwrap()  {
                if entry["request"]["url"].as_str().unwrap().eq("https://binusmaya.binus.ac.id/services/ci/index.php/general/getBinusianData") {
                    let binusian_data: models::user::BinusianData = serde_json::from_str(entry["response"]["content"]["text"].as_str().unwrap().clone()).unwrap();
                    let user_binusian_data = models::user::UserBinusianData::init(&binusian_data);
                    let old_binusmaya_api = OldBinusmayaAPI::login(&user_binusian_data, user_credential).await;
                    cookie = Some(old_binusmaya_api.cookie);
                    break;
                }
            }
        }

        selenium.quit().await?;
    
        Ok(CookieOutput::Out(is_valid, cookie))
    } else {
        let cookie = None;

        selenium.quit().await?;
    
        Ok(CookieOutput::Out(is_valid, cookie))
    }
}

async fn add_new_binusmaya_account(
    repository: &repository::new_binusmaya_repository::NewBinusmayaRepository,
    user_credential: models::user::UserCredential,
    msg: &Message,
    ctx: &Context,
) -> CommandResult {
    let proxy = BrowserMobProxy {
        host: "192.168.100.25",
        port: 20000,
    };

    let handle = tokio::task::spawn(async move {
        launch_selenium(&user_credential, &proxy, NEW_BINUSMAYA.to_string()).await.unwrap()
    })
    .await?;

    match handle {
        CookieOutput::Out(Status::VALID(output), _) => {
            let har = proxy.get_har().await?;
            let len = har["log"]["entries"].as_array().unwrap().len();
            let bearer_token =
                &har["log"]["entries"][len - 1]["request"]["headers"][6]["value"].to_string();
                
            let user_record = &models::user::NewBinusmayaUser {
                member_id: *msg.author.id.as_u64(),
                auth: bearer_token[1..bearer_token.len() - 1].to_string(),
                last_registered: Local::now(),
            };
            
            repository.insert(user_record).unwrap();

            msg.author
                .dm(&ctx, |m| {
                    m.embed(|e| {
                        e.colour(PRIMARY_COLOR)
                            .field("Account Registered", output, false)
                    })
                })
                .await?;
        }
        CookieOutput::Out(Status::INVALID(output), _) => {
            msg.author
                .dm(&ctx, |m| {
                    m.embed(|e| {
                        e.colour(PRIMARY_COLOR)
                            .field("Account is not valid", output, false)
                    })
                })
                .await?;
        }
        CookieOutput::Out(Status::ERROR(output), _) => {
            msg.author
                .dm(&ctx, |m| {
                    m.embed(|e| {
                        e.colour(PRIMARY_COLOR)
                            .field("Error in registering", output, false)
                    })
                })
                .await?;
        }
    }

    proxy.delete_proxy().await?;

    Ok(())
}

async fn add_old_binusmaya_account(
    repository: &repository::old_binusmaya_repository::OldBinusmayaRepository,
    user_credential: models::user::UserCredential,
    msg: &Message,
    ctx: &Context,
) -> CommandResult {
    let proxy = BrowserMobProxy {
        host: "192.168.100.25",
        port: 8082,
    };

    let user_credential_clone = user_credential.clone();

    let handle = tokio::task::spawn(async move {
        launch_selenium(&user_credential_clone.clone(), &proxy, OLD_BINUSMAYA.to_string()).await.unwrap()
    })
    .await?;

    match handle {
        CookieOutput::Out(Status::VALID(output), cookie) => {
            let cookie_clone = cookie.clone();
            let old_binusmaya_api = OldBinusmayaAPI {
                cookie: cookie_clone.unwrap()
            };
            let binusian_data = old_binusmaya_api.get_binusian_data().await?;
            let user_binusian_data = models::user::UserBinusianData::init(&binusian_data);

            let encrypted_user_credential = models::user::UserCredential {
                email: user_credential.email.clone(),
                password: MAGIC_CRYPT.encrypt_str_to_base64(user_credential.password.clone())
            };

            let user_record = models::user::OldBinusmayaUser {
                member_id: *msg.author.id.as_u64(),
                user_credential: encrypted_user_credential,
                binusian_data: user_binusian_data,
                cookie: String::from(""),
            };

            repository.insert(&user_record).unwrap();

            msg.author
                .dm(&ctx, |m| {
                    m.embed(|e| {
                        e.colour(PRIMARY_COLOR)
                            .field("Account Registered", output, false)
                    })
                })
                .await?;
        }
        CookieOutput::Out(Status::INVALID(output), _) => {
            msg.author
                .dm(&ctx, |m| {
                    m.embed(|e| {
                        e.colour(PRIMARY_COLOR)
                            .field("Account is not valid", output, false)
                    })
                })
                .await?;
        }
        CookieOutput::Out(Status::ERROR(output), _) => {
            msg.author
                .dm(&ctx, |m| {
                    m.embed(|e| {
                        e.colour(PRIMARY_COLOR)
                            .field("Error in registering", output, false)
                    })
                })
                .await?;
        }
    }

    proxy.delete_proxy().await?;

    Ok(())
}

#[command]
#[only_in("dm")]
#[description("Add BINUS account to discord bot")]
#[usage("[email];[password]")]
async fn add(ctx: &Context, msg: &Message) -> CommandResult {
    msg.react(&ctx, '👍').await?;

    let binus_ver = msg.channel_id.send_message(&ctx, |m| {
        m.embed(|e| e
            .colour(PRIMARY_COLOR)
            .field("Choose Binusmaya Version", "Please choose Binusmaya version you want to register to.", false)
        );
        m.components(|c| c.add_action_row(Binusmaya::action_row()))
    }).await?;

    let mci = match binus_ver.await_component_interaction(&ctx).timeout(Duration::minutes(1).to_std().unwrap()).await {
        Some(ci) => ci,
        None => {
            msg.reply(&ctx, "Timed Out, please try again").await?;
            return Ok(());
        }
    };

    let binusmaya_version = Binusmaya::from_str(&mci.data.values.get(0).unwrap()).unwrap();

    mci.create_interaction_response(&ctx, |r| {
        r.kind(InteractionResponseType::ChannelMessageWithSource);
        r.interaction_response_data(|d| 
            d.content("Please enter your Binus email and password with format of `[email] [password]`")
        )
    }).await?;

    if let Some(reply) = msg.author.await_reply(&ctx).await {
        let user_credential: Vec<&str> = reply.content.split(' ').collect();
        if user_credential.len() == 2 {
            let user_credential = models::user::UserCredential {
                email: user_credential[0].to_string(),
                password: user_credential[1].to_string()
            };

            match binusmaya_version {
                Binusmaya::NewBinusmaya=> {
                    let user_data = consts::NEW_BINUSMAYA_REPO.get_by_id(msg.author.id.as_u64());
                    if user_data.as_ref().is_some_and(|user| user.is_ok()) {
                        let jwt_exp = user_data.unwrap()?.last_registered.add(Duration::days(7));
                        let now = chrono::offset::Local::now();

                        if jwt_exp < now {
                            msg.channel_id
                                .send_message(&ctx, |m| {
                                    m.embed(|e| {
                                        e.colour(PRIMARY_COLOR).field(
                                            "Registering...",
                                            "Please wait a few seconds",
                                            false,
                                        )
                                    })
                                })
                                .await?;

                            add_new_binusmaya_account(&consts::NEW_BINUSMAYA_REPO, user_credential, msg, ctx).await.unwrap();
                        } else {
                            msg.channel_id
                                .send_message(&ctx, |m| {
                                    m.embed(|e| {
                                        e.colour(PRIMARY_COLOR).field(
                                            "You've already registered",
                                            format!(
                                                "Please wait **{} days** to re-register your account",
                                                jwt_exp.signed_duration_since(now).num_days()
                                            ),
                                            false,
                                        )
                                    })
                                })
                                .await?;
                        }
                    } else {
                        msg.channel_id
                            .send_message(&ctx, |m| {
                                m.embed(|e| {
                                    e.colour(PRIMARY_COLOR).field(
                                        "Registering...",
                                        "Please wait a few seconds",
                                        false,
                                    )
                                })
                            }
                        )
                        .await?;

                        add_new_binusmaya_account(&consts::NEW_BINUSMAYA_REPO, user_credential, msg, ctx).await.unwrap();
                    }
                },
                Binusmaya::OldBinusmaya => {
                    msg.channel_id
                        .send_message(&ctx, |m| {
                            m.embed(|e| {
                                e.colour(PRIMARY_COLOR).field(
                                    "Registering...",
                                    "Please wait a few seconds",
                                    false,
                                )
                            })
                        }
                    ).await?;

                    add_old_binusmaya_account(&consts::OLD_BINUSMAYA_REPO, user_credential, msg, ctx).await.unwrap();            
                }
            }

        } else {
            msg.channel_id.send_message(&ctx, |m| {
                m.embed(|e| e
                    .colour(PRIMARY_COLOR)
                    .field("Error", "Missing email or password", false)
                )
            }).await?;

            return Ok(());
        }
    }
       
    Ok(())
}
