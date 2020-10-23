use clap::ArgMatches;
use scraper::{Html, Selector, ElementRef};
use regex::Regex;
use rand::thread_rng;
use rand::seq::SliceRandom;
use colored::Colorize;
use std::io::{self, Write};
use crate::config::Config;
use crate::kattis_client::KattisClient;
use crate::commands::get::get_and_create_problem;
use crate::StdErr;

pub async fn random(cmd: &ArgMatches<'_>) -> Result<(), StdErr> {
    let cfg = Config::load()?;
    let host_name = cfg.get_host_name()?;

    let problems = get_front_page_problems(cmd, &cfg).await?;
    let mut rng = thread_rng();
    let problem = match problems.choose(&mut rng) {
        Some(p) => p,
        None => return Err("could not find a random problem".into())
    };

    println!("{}:    {}", "Problem".bright_cyan(), problem.id);
    println!("{}: {}", "Difficulty".bright_cyan(), problem.difficulty);
    println!("{}:        {}", "URL".bright_cyan(), format!("https://{}/problems/{}", &host_name, problem.id).underline());

    if !cmd.is_present("yes") {
        print!("Do you want to fetch this problem? (y/n): ");
        io::stdout().flush().expect("failed to flush stdout");

        let mut input = String::new();
        if let Err(_) = io::stdin().read_line(&mut input) {
            return Err("failed to read input".into());
        }

        if input.trim().to_lowercase() != "y" {
            return Ok(());
        }
    }

    get_and_create_problem(&problem.id, host_name).await?;

    Ok(())
}

#[derive(Debug)]
struct SimpleProblem {
    name: String,
    id: String,
    difficulty: String,
}

async fn get_front_page_problems(cmd: &ArgMatches<'_>, cfg: &Config) -> Result<Vec<SimpleProblem>, StdErr> {
    let creds = cfg.get_credentials()?;
    let login_url = cfg.get_login_url()?;
    let host_name = cfg.get_host_name()?;

    let sort_by_arg = cmd.value_of("sort").unwrap();
    let sort_by = match sort_by_arg {
        "total" => "subtot",
        "acc" => "subacc",
        "ratio" => "subrat",
        "difficulty" => "problem_difficulty",
        _ => sort_by_arg,
    };

    let kc = KattisClient::new()?;
    let query = format!("order={}&dir={}&show_solved=off&show_tried=off&show_untried=on",
                        sort_by, cmd.value_of("direction").unwrap());
    let url = format!("https://{}/problems?{}", host_name, query);

    kc.login(creds, login_url).await?;
    let res = kc.client.get(&url).send().await?;

    let status = res.status();
    if !status.is_success() {
        return Err(format!("failed to get problems from kattis (http status code {})", status).into());
    }

    let html = match res.text().await {
        Ok(h) => h,
        Err(_) => return Err("failed to read response from kattis".into()),
    };

    let doc = Html::parse_document(&html);
    let row_selector = Selector::parse(".problem_list > tbody > tr").unwrap();
    let title_selector = Selector::parse(".name_column > a").unwrap();
    let numeric_selector = Selector::parse(".numeric").unwrap();
    let id_regex = Regex::new(r"/(\w+)$").unwrap();

    let mut problems = Vec::new();
    for row in doc.select(&row_selector) {
        let name_el = match row.select(&title_selector).next() {
            Some(t) => t,
            None => continue,
        };
        let name: String = name_el.text().collect();
        let id_match = name_el.value()
            .attr("href")
            .and_then(|a| id_regex.captures(a))
            .and_then(|c| c.get(1));
        let id = match id_match {
            Some(m) => m.as_str().to_string(),
            None => continue,
        };

        let cols: Vec<ElementRef> = row.select(&numeric_selector).collect();
        let difficulty: String = match cols.last() {
            Some(l) => l.text().collect(),
            None => continue,
        };

        problems.push(SimpleProblem { name, id, difficulty });
    }

    Ok(problems)
}
