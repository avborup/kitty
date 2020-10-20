use reqwest::{Client, Response};
use reqwest::multipart::Form;
use crate::config::Credentials;
use crate::StdErr;

pub const USER_AGENT: &'static str = env!("CARGO_PKG_NAME");

pub struct KattisClient {
    pub client: Client,
}

impl KattisClient {
    pub fn new() -> Result<Self, StdErr> {
        let client = Client::builder()
            .cookie_store(true)
            .user_agent(USER_AGENT)
            .build()?;

        Ok(Self { client })
    }

    pub async fn login(&self, creds: Credentials, login_url: &str) -> Result<Response, StdErr> {
        let form = Form::new()
            .text("user", creds.username)
            .text("token", creds.token)
            .text("script", "true");
        let res = self.client.post(login_url)
            .multipart(form)
            .send()
            .await?;

        let status = res.status();
        if !status.is_success() {
            match res.status().as_str() {
                "403" => return Err("the login credentials from your .kattisrc are not valid".into()),
                _ => return Err(format!("failed to log in to kattis (http status code {})", status).into()),
            }
        }

        Ok(res)
    }
}
