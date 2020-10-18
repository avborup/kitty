use clap::ArgMatches;
use platform_dirs::AppDirs;
use ini::Ini;
use reqwest::{Client, Response};
use reqwest::multipart::{Form, Part};
use std::fs;
use std::io::{self, Write};
use regex::Regex;
use colored::Colorize;
use crate::StdErr;
use crate::problem::Problem;

const USER_AGENT: &'static str = env!("CARGO_PKG_NAME");

pub async fn submit(cmd: &ArgMatches<'_>) -> Result<(), StdErr> {
    let problem = Problem::from_args(cmd)?;

    let file_path = problem.file();
    let file_name = match file_path.file_name() {
        Some(f) => f,
        None => return Err("failed to get file name".into()),
    }.to_str().expect("file path contained invalid unicode");
    
    if !cmd.is_present("yes") {
        println!("{}:  {}", "Problem".bright_cyan(), problem.name());
        println!("{}: {}", "Language".bright_cyan(), problem.lang());
        println!("{}:     {}", "File".bright_cyan(), file_name);
        print!("Is this correct? (y/n): ");
        io::stdout().flush().expect("failed to flush stdout");

        let mut input = String::new();
        if let Err(_) = io::stdin().read_line(&mut input) {
            return Err("failed to read input".into());
        }

        if input.trim().to_lowercase() != "y" {
            return Ok(());
        }
    }

    let creds = get_credentials()?;

    let client = Client::builder()
        .cookie_store(true)
        .user_agent(USER_AGENT)
        .build()?;

    login(&client, creds).await?;

    let id = match submit_problem(&client, &problem).await? {
        Some(i) => i,
        None => return Err("something went wrong during submission".into()),
    };

    println!("{} solution successfully. view it at https://open.kattis.com/submissions/{}", "submitted".bright_green(), id);

    Ok(())
}

struct Credentials {
    username: String,
    token: String,
}

fn get_credentials() -> Result<Credentials, StdErr> {
    let app_dirs = match AppDirs::new(Some("kitty"), false) {
        Some(a) => a,
        None => return Err("failed to find kitty config directory".into()),
    };
    let config_path = app_dirs.config_dir.join(".kattisrc");

    if !config_path.exists() {
        return Err(format!("could not find .kattisrc file. you must download your .kattisrc file from https://open.kattis.com/download/kattisrc and save it at {}",
                           config_path.to_str().expect("config file path contained invalid unicode")).into());
    }

    let cfg = match Ini::load_from_file(config_path) {
        Ok(c) => c,
        Err(_) => return Err("failed to read .kattisrc file".into()),
    };

    let user_section = match cfg.section(Some("user")) {
        Some(u) => u,
        None => return Err("could not find user section in .kattisrc".into()),
    };

    let username = match user_section.get("username") {
        Some(u) => u,
        None => return Err("could not find username under [user] in .kattisrc".into()),
    };

    let token = match user_section.get("token") {
        Some(u) => u,
        None => return Err("could not find token under [user] in .kattisrc".into()),
    };

    Ok(Credentials {
        username: username.to_string(),
        token: token.to_string(),
    })
}

async fn login(client: &Client, creds: Credentials) -> Result<Response, StdErr> {
    let form = Form::new()
        .text("user", creds.username)
        .text("token", creds.token)
        .text("script", "true");
    let res = client.post("https://open.kattis.com/login")
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

async fn submit_problem(client: &Client, problem: &Problem) -> Result<Option<String>, StdErr> {
    let file_path = problem.file();
    let file_name = file_path.file_name().unwrap().to_str().unwrap().to_string();

    let file_bytes = match fs::read(&file_path) {
        Ok(b) => b,
        Err(_) => return Err("failed to read solution file".into())
    };
    let file_part = Part::bytes(file_bytes)
        .file_name(file_name)
        .mime_str("application/octet-stream")
        .expect("failed to set mime type for file");

    let form = Form::new()
        .text("problem", problem.name())
        .text("language", problem.lang().to_string())
        .text("mainclass", problem.get_main_class().unwrap_or(String::new()))
        .part("sub_file[]", file_part)
        .text("submit_ctr", "2")
        .text("submit", "true")
        .text("script", "true");

    let res = client.post("https://open.kattis.com/submit")
        .multipart(form)
        .send()
        .await?;

    let status = res.status();
    if !status.is_success() {
        return Err(format!("failed to submit to kattis (http status code {})", status).into());
    }

    let content = match res.text().await {
        Ok(t) => t,
        Err(_) => return Err("failed to read response from kattis".into()),
    };

    if content.contains("Problem not found") {
        return Err(format!("the problem \"{}\" does not exist", problem.name()).into());
    }

    let re = Regex::new(r"ID: (\d+)").unwrap();
    let id = re.captures(&content)
        .and_then(|c| c.get(1))
        .and_then(|i| Some(i.as_str().to_string()));

    Ok(id)
}
