use chrono::{Duration, Local};
use csv_async::AsyncWriterBuilder;
use magic_crypt::MagicCryptTrait;
use serenity::{
    framework::standard::{macros::command, CommandResult},
    model::prelude::*,
    prelude::*, builder::{CreateSelectMenuOption, CreateSelectMenu, CreateActionRow},
};
use std::{io::Write, str::FromStr, fmt};
use std::ops::Add;
use std::{error::Error, fs::OpenOptions};
use thirtyfour::{error::WebDriverError, Capabilities, DesiredCapabilities, Proxy, WebDriver, Cookie};

use crate::{
    consts::{CHROME_BINARY, NEW_BINUSMAYA, PRIMARY_COLOR, NEWBINUSMAYA_USER_DATA, NEWBINUSMAYA_USER_FILE, OLD_BINUSMAYA, OLDBINUSMAYA_USER_DATA, OLDBINUSMAYA_USER_FILE, CHROME_SERVER_URL, MAGIC_CRYPT},
    discord::{discord::{NewBinusmayaUserAuthInfo, NewBinusmayaUserRecord, UserCredential, OldBinusmayaUserRecord}, helper::ParseError},
    api::dropbox_api,
    third_party::{BrowserMobProxy, Selenium, Status},
};

#[derive(Debug)]
enum CookieOutput<T, C> {
	Out(T, C)
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
    user_credential: &UserCredential,
    proxy: &BrowserMobProxy,
    binus_ver: String,
) -> Result<CookieOutput<Status<String>, Cookie>, WebDriverError> {
    proxy.create_proxy().await?;

    let proxy_port = proxy.get_proxy().await?;
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
    caps.accept_ssl_certs(true)?;
    caps.set_binary(CHROME_BINARY.lock().await.as_str())?;
    caps.add_chrome_arg("--proxy-server=http://localhost:8083")?;
    caps.add_chrome_arg("--ignore-certificate-errors")?;
    caps.set_headless()?;

    let selenium = Selenium::init(
        WebDriver::new(CHROME_SERVER_URL, &caps).await?,
        user_credential.email.clone(),
        user_credential.password.clone(),
    );

    selenium.setup().await?;

    BrowserMobProxy::new_har(&proxy).await?;
    let is_valid = selenium.run(&binus_ver.to_string()).await.unwrap_or(
		Status::ERROR("Error in registering, please try again. If the problem still persist, please contact `PlayerPlay#9549` or open a new issue [here](https://github.com/BryanEgbert/BINUSMAYA_Discord_Bot/issues)".to_string())
	);

    let cookie = selenium.get_cookie().await?;

    selenium.quit().await?;

    Ok(CookieOutput::Out(is_valid, cookie))
}

async fn write_user_data(
    binus_ver: &Binusmaya,
    proxy: &BrowserMobProxy,
    msg: &Message,
    user_credential: &UserCredential,
    cookie: Cookie
) -> Result<(), Box<dyn Error>> {
    match binus_ver {
        Binusmaya::NewBinusmaya => {
            let har = BrowserMobProxy::get_har(&proxy).await?;
            let len = har["log"]["entries"].as_array().unwrap().len();
            let bearer_token =
                &har["log"]["entries"][len - 1]["request"]["headers"][6]["value"].to_string();
            
            let user_data = NEWBINUSMAYA_USER_DATA.clone();
        
            let user_record = &NewBinusmayaUserRecord {
                member_id: *msg.author.id.as_u64(),
                auth: bearer_token[1..bearer_token.len() - 1].to_string(),
                last_registered: Local::now(),
            };
        
            user_data.lock().await.insert(
                user_record.member_id,
                NewBinusmayaUserAuthInfo {
                    auth: user_record.auth.clone(),
                    last_registered: user_record.last_registered,
                },
            );
        
            let mut wtr = AsyncWriterBuilder::new()
                .has_headers(false)
                .create_serializer(vec![]);
        
            wtr.serialize(user_record).await?;
        
            let mut file = OpenOptions::new()
                .append(true)
                .open(NEWBINUSMAYA_USER_FILE)
                .unwrap();
        
            if let Err(err) = write!(
                file,   
                "{}",
                String::from_utf8(wtr.into_inner().await?).unwrap()
            ) {
                eprintln!("Error when writing to a file: {}", err);
            }
        
            let res = dropbox_api::upload_file(NEWBINUSMAYA_USER_FILE.to_string()).await?;
            println!("File upload status code: {}", res);
        },
        Binusmaya::OldBinusmaya => {
            let encrypted_user_credential = UserCredential {
                email: user_credential.email.clone(),
                password: MAGIC_CRYPT.encrypt_str_to_base64(user_credential.password.clone())
            };

            let user_record = OldBinusmayaUserRecord {
                member_id: *msg.author.id.as_u64(),
                user_credential: encrypted_user_credential
            };

            let user_data = OLDBINUSMAYA_USER_DATA.clone();
            user_data.lock().await.insert(*msg.author.id.as_u64(), cookie);

            let mut wtr = AsyncWriterBuilder::new()
                .has_headers(false)
                .create_serializer(vec![]);

            wtr.serialize(user_record).await?;

            let mut file = OpenOptions::new()
                .append(true)
                .open(OLDBINUSMAYA_USER_FILE)
                .unwrap();
            
            if let Err(err) = write!(file, "{}", String::from_utf8(wtr.into_inner().await?).unwrap()) {
                eprintln!("Error when writing to a file: {}", err);
            }
                
        },
    }

    Ok(())
}

async fn add_account(
    user_credential: UserCredential,
    binus_ver: Binusmaya,
    msg: &Message,
    ctx: &Context,
) -> CommandResult {
    let proxy = BrowserMobProxy {
        host: "localhost",
        port: 8082,
    };

    let binusmaya_ver = binus_ver.clone();
    let user_credential_clone = user_credential.clone();

    let handle = tokio::task::spawn(async move {
        match binusmaya_ver {
            Binusmaya::NewBinusmaya => launch_selenium(&user_credential_clone, &proxy, NEW_BINUSMAYA.to_string()).await.unwrap(),
            Binusmaya::OldBinusmaya =>  launch_selenium(&user_credential_clone.clone(), &proxy, OLD_BINUSMAYA.to_string()).await.unwrap()
        }
    })
    .await?;

    match handle {
        CookieOutput::Out(Status::VALID(output), cookie) => {
            match binus_ver {
                Binusmaya::NewBinusmaya => {
                    write_user_data(&binus_ver, &proxy, &msg, &user_credential, cookie).await.unwrap();
    
                    msg.author
                        .dm(&ctx, |m| {
                            m.embed(|e| {
                                e.colour(PRIMARY_COLOR)
                                    .field("Account Registered", output, false)
                            })
                        })
                        .await?;
                },
                Binusmaya::OldBinusmaya => {
                    write_user_data(&binus_ver, &proxy, &msg, &user_credential, cookie).await.unwrap();

                    msg.author
                        .dm(&ctx, |m| {
                            m.embed(|e| {
                                e.colour(PRIMARY_COLOR)
                                    .field("Account Registered", output, false)
                            })
                        })
                        .await?;
                },
            }
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
            let user_credential = UserCredential {
                email: user_credential[0].to_string(),
                password: user_credential[1].to_string()
            };

            match binusmaya_version {
                Binusmaya::NewBinusmaya=> {
                    if NEWBINUSMAYA_USER_DATA.lock().await.contains_key(msg.author.id.as_u64()) {
                        let jwt_exp = NEWBINUSMAYA_USER_DATA
                            .lock()
                            .await
                            .get(msg.author.id.as_u64())
                            .unwrap()
                            .last_registered
                            .add(Duration::weeks(52));
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

                    add_account(user_credential, binusmaya_version, msg, ctx).await.unwrap();            
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


    // 
       
    Ok(())
}
