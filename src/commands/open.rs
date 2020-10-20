use clap::ArgMatches;
use std::env;
use colored::Colorize;
use crate::config::Config;
use crate::StdErr;

pub async fn open(cmd: &ArgMatches<'_>) -> Result<(), StdErr> {
    let id = match cmd.value_of("PROBLEM ID") {
        Some(s) => {
            let mut s = s.to_string();
            s.retain(char::is_alphanumeric);

            s
        },
        None => {
            let cwd = match env::current_dir() {
                Ok(d) => d,
                Err(_) => return Err("failed to get current directory".into()),
            };

            let dir_name = match cwd.file_name() {
                Some(n) => n,
                None => return Err("failed to get name of current directory".into()),
            };

            dir_name.to_str().expect("directory name contained invalid unicode").to_string()
        },
    };

    if id.len() == 0 {
        return Err(format!("\"{}\" is not a valid problem id", &id).into());
    }

    let cfg = Config::load()?;
    let host_name = cfg.get_host_name()?;
    let url = format!("https://{}/problems/{}", host_name, &id);

    if let Err(_) = webbrowser::open(&url) {
        return Err(format!("failed to open {} in your browser", &url).into());
    }

    println!("{} {} in your browser", "opened".bright_green(), &url.underline());

    Ok(())
}
