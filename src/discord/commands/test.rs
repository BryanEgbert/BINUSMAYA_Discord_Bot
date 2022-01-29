use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::prelude::*,
    prelude::*,
};
use thirtyfour::{error::WebDriverError, DesiredCapabilities, WebDriver, Cookie};

use crate::{
    consts::OLD_BINUSMAYA,
    third_party::{Selenium, Status},
};

pub enum CookieOutput<T, C> {
	Out(T, C)
}

async fn launch_selenium(
    email: String,
    password: String,
) -> Result<CookieOutput<Status<String>, Option<Vec<Cookie>>>, WebDriverError> {
    let caps = DesiredCapabilities::chrome();
    let selenium = Selenium::init(
        WebDriver::new("http://localhost:4444", &caps).await?,
        email.clone(),
        password.clone(),
    );

	let cookie;

    selenium.setup().await?;

    let is_valid = selenium.run(&OLD_BINUSMAYA.to_string()).await?;

	if let Status::VALID(_) = is_valid {
		cookie = Some(selenium.get_cookies().await?);
	} else {
		cookie = None;
	}

    selenium.quit().await?;

    Ok(CookieOutput::Out(is_valid, cookie))
}

#[command]
#[num_args(2)]
async fn test(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let email = args.single::<String>().unwrap();
    let password = args.single::<String>().unwrap();

    msg.react(&ctx, '👍').await?;

    let handle = tokio::task::spawn(async move {
        launch_selenium(email.clone(), password.clone())
            .await
            .unwrap()
    })
    .await?;

    match handle {
        CookieOutput::Out(Status::VALID(_), cookies) => {
            println!("{:#?}", cookies);
        }
        CookieOutput::Out(Status::INVALID(_), _) => {}
        CookieOutput::Out(Status::ERROR(_), _) => {}
    }

    Ok(())
}
