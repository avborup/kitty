use crate::config::Config;
use crate::kattis_client::KattisClient;
use crate::StdErr;
use clap::ArgMatches;
use scraper::{Html, Selector};

const CHECKBOX: &'static str = "\u{2705}"; // Green checkbox emoji
const CROSSMARK: &'static str = "\u{274C}"; // Red X emoji

pub async fn history(cmd: &ArgMatches<'_>) -> Result<(), StdErr> {
    let count = match cmd.value_of("count").unwrap().parse() {
        Ok(n) => n,
        Err(_) => return Err("please provide --count as an integer".into()),
    };

    let cfg = Config::load()?;
    let kc = KattisClient::new()?;
    let history = get_history(&kc, &cfg).await?;

    let n = if cmd.is_present("all") {
        history.len()
    } else {
        count
    };
    for submission in history.iter().take(n) {
        println!("{} {}", submission.status_symbol, submission.title);
    }

    Ok(())
}

struct Submission {
    title: String,
    status_symbol: String,
}

async fn get_history(kc: &KattisClient, cfg: &Config) -> Result<Vec<Submission>, StdErr> {
    let creds = cfg.get_credentials()?;
    let login_url = cfg.get_login_url()?;
    let profile_url = format!("https://{}/users/{}", cfg.get_host_name()?, creds.username);

    kc.login(creds.clone(), login_url).await?;
    let res = kc.client.get(&profile_url).send().await?;

    let status = res.status();
    if !status.is_success() {
        return Err(format!(
            "failed to retrieve history from kattis (http status code {})",
            status
        )
        .into());
    }

    let html = match res.text().await {
        Ok(h) => h,
        Err(_) => return Err("failed to read history response from kattis".into()),
    };

    let doc = Html::parse_document(&html);
    let row_selector = Selector::parse(".table-submissions > tbody > tr").unwrap();
    let title_selector = Selector::parse("#problem_title").unwrap();
    let success_selector = Selector::parse("td[data-type='status'] > .accepted").unwrap();

    let mut submissions = Vec::new();
    for row in doc.select(&row_selector) {
        let title: String = match row.select(&title_selector).next() {
            Some(t) => t.text().collect(),
            None => continue,
        };
        let status_symbol = if row.select(&success_selector).next().is_some() {
            CHECKBOX
        } else {
            CROSSMARK
        }
        .to_string();

        submissions.push(Submission {
            title,
            status_symbol,
        });
    }

    Ok(submissions)
}
