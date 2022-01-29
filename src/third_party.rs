use serde::Deserialize;
use serde_json::json;
use std::time::Duration;
use thirtyfour::{extensions::chrome::ChromeDevTools, prelude::*};
use tokio;

use crate::consts::NEW_BINUSMAYA;

#[derive(Clone, Copy)]
pub struct BrowserMobProxy {
    pub host: &'static str,
    pub port: u16,
}

pub struct Selenium {
    pub driver: WebDriver,
    pub email: String,
    pub password: String,
}

#[derive(Debug)]
pub enum Status<T> {
    VALID(T),
    INVALID(T),
    ERROR(T),
}

#[derive(Deserialize)]
pub struct Port {
    pub port: u32,
}

#[derive(Deserialize)]
pub struct ProxyList {
    pub proxyList: Vec<Port>,
}

impl BrowserMobProxy {
    pub async fn create_proxy(&self) -> Result<reqwest::StatusCode, reqwest::Error> {
        let client = reqwest::Client::new();
        let res = client
            .post(format!("http://{}:{}/proxy", self.host, self.port))
            .query(&[("trustAllServers", "true")])
            .send()
            .await?;

        Ok(res.status())
    }

    pub async fn get_proxy(&self) -> Result<ProxyList, reqwest::Error> {
        let res: ProxyList = reqwest::get(format!("http://{}:{}/proxy", self.host, self.port))
            .await?
            .json()
            .await?;

        Ok(res)
    }

    pub async fn new_har(&self) -> Result<reqwest::StatusCode, reqwest::Error> {
        let client = reqwest::Client::new();
        let proxy = self.get_proxy().await?;
        let index = proxy.proxyList.len() - 1;

        let res = client
            .put(format!(
                "http://{}:{}/proxy/{}/har",
                self.host,
                self.port,
                proxy.proxyList[index].port.to_string()
            ))
            .query(&[
                ("captureHeaders", "true"),
                ("initialPageTitle", "newbinusmaya"),
            ])
            .send()
            .await?;

        Ok(res.status())
    }

    pub async fn get_har(&self) -> Result<serde_json::Value, reqwest::Error> {
        let proxy = self.get_proxy().await?;
        let res = reqwest::get(format!(
            "http://{}:{}/proxy/{}/har",
            self.host,
            self.port,
            proxy.proxyList[proxy.proxyList.len() - 1].port
        ))
        .await?
        .json()
        .await?;

        Ok(res)
    }

    pub async fn delete_proxy(&self) -> Result<reqwest::StatusCode, reqwest::Error> {
        let proxy = self.get_proxy().await?;
        let client = reqwest::Client::new();
        let res = client
            .delete(format!(
                "http://{}:{}/proxy/{}",
                self.host, self.port, proxy.proxyList[0].port
            ))
            .send()
            .await?;

        Ok(res.status())
    }
}

impl Selenium {
    async fn microsoft_login(&self) -> WebDriverResult<Status<String>> {
		tokio::time::sleep(Duration::from_millis(1000)).await;

        let mcr_email = self
            .driver
            .find_element(By::Id("i0116"))
            .await
            .expect("Error in getting mcr_email");

        tokio::time::sleep(Duration::from_millis(1000)).await;

        mcr_email
            .send_keys(TypingData::from(self.email.clone()) + Keys::Enter)
            .await?;
        tokio::time::sleep(Duration::from_millis(1000)).await;

        self.driver
            .find_element(By::Id("i0118"))
            .await?
            .send_keys(TypingData::from(self.password.clone()) + Keys::Enter)
            .await
            .expect("Error in typing password");
        self.driver
            .find_element(By::Id("idSIButton9"))
            .await?
            .click()
            .await
            .expect("Error in getting mcr_btn");
        tokio::time::sleep(Duration::from_millis(1000)).await;

        if self
            .driver
            .current_url()
            .await?
            .contains("https://login.microsoftonline.com/")
        {
            Ok(Status::INVALID("Wrong email or password".to_string()))
        } else {
            Ok(Status::VALID("Account successfully registered".to_string()))
        }
    }

    pub fn init(driver: WebDriver, email: String, password: String) -> Self {
        Self {
            driver,
            email,
            password,
        }
    }

    pub async fn setup(&self) -> WebDriverResult<()> {
        self.driver
            .set_implicit_wait_timeout(Duration::new(30, 0))
            .await?;

        Ok(())
    }

    pub async fn run(&self, link: &String) -> WebDriverResult<Status<String>> {
        let dev_tools = ChromeDevTools::new(&self.driver);
        dev_tools.execute_cdp("Network.enable").await?;
        dev_tools.execute_cdp_with_params(
			"Network.setBlockedURLs", 
		json!({"urls": vec!["*.jpg", "*.woff2", "*.woff", "*.ttf", "*.svg", "*.jpeg", "*.png", "*.dahsboard", "*func-bm7-notification-prod*", "*.ico", "*.json", "*image/*", "*func-bm7-organization-prod*", "*ToDo*", "*func-bm7-forum-prod*", "*fonts.googleapis.com*", 
			"https://apim-bm7-prod.azure-api.net/func-bm7-course-prod/ClassSession/Ongoing/student", "https://apim-bm7-prod.azure-api.net/func-bm7-course-prod/Session/AcademicPeriod/2110"]
		})).await?;

        self.driver.get(link).await?;

        if link.eq(&NEW_BINUSMAYA) {
            let mcr_login_btn = self
                .driver
                .find_element(By::ClassName("button--azure--ad"))
                .await
                .expect("Error in getting mcr_login_btn");
            mcr_login_btn.click().await?;
        } else {
            let login_btn = self
                .driver
                .find_element(By::XPath(
                    "/html/body/div/section[2]/div/div[2]/div/div[2]/div[1]/a[2]",
                ))
                .await;
            if let Ok(login_btn) = login_btn {
                login_btn.click().await?;
            } else {
                return Ok(Status::ERROR("Website under maintenence".to_string()));
            }
        }

        let status = Selenium::microsoft_login(&self).await?;

        tokio::time::sleep(Duration::from_millis(10000)).await;

        Ok(status)
    }

    pub async fn get_cookies(&self) -> WebDriverResult<Vec<Cookie>> {
        Ok(self.driver.get_cookies().await?)
    }

    pub async fn quit(self) -> WebDriverResult<()> {
        self.driver.quit().await?;

        Ok(())
    }
}