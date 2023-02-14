use std::ops::Deref;

use eyre::Context;
use reqwest::{multipart::Form, Client, StatusCode};
use secrecy::ExposeSecret;

use crate::App;

pub const USER_AGENT: &str = env!("CARGO_PKG_NAME");

#[derive(Debug)]
pub struct KattisClient {
    pub client: Client,
}

impl Deref for KattisClient {
    type Target = Client;

    fn deref(&self) -> &Self::Target {
        &self.client
    }
}

impl KattisClient {
    pub fn new() -> crate::Result<Self> {
        let client = Client::builder()
            .cookie_store(true)
            .user_agent(USER_AGENT)
            .build()
            .wrap_err("Failed to instantiate HTTP client")?;

        Ok(Self { client })
    }

    pub async fn login(&self, app: &App) -> crate::Result<()> {
        let kattisrc = app.config.try_kattisrc()?;

        let form = Form::new()
            .text("user", kattisrc.user.username.clone())
            .text("token", kattisrc.user.token.expose_secret().clone())
            .text("script", "true");

        let res = self
            .client
            .post(&kattisrc.kattis.login_url)
            .multipart(form)
            .send()
            .await
            .wrap_err("Failed to send login request to Kattis")?;

        if res.status() == StatusCode::FORBIDDEN {
            eyre::bail!(
                "Invalid username/token for Kattis. Please check your .kattisrc credentials."
            )
        }

        if !res.status().is_success() {
            eyre::bail!(
                "Failed to log in to Kattis (http status code {})",
                res.status()
            );
        }

        Ok(())
    }
}
