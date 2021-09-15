use crate::problem::Problem;
use crate::StdErr;
use crate::CFG as cfg;
use clap::ArgMatches;
use colored::Colorize;
use std::env;

pub async fn open(cmd: &ArgMatches<'_>) -> Result<(), StdErr> {
    let id = match cmd.value_of("PROBLEM ID") {
        Some(s) => s.to_string(),
        None => {
            let cwd = match env::current_dir() {
                Ok(d) => d,
                Err(_) => return Err("failed to get current directory".into()),
            };

            let dir_name = match cwd.file_name() {
                Some(n) => n,
                None => return Err("failed to get name of current directory".into()),
            };

            dir_name
                .to_str()
                .expect("directory name contained invalid unicode")
                .to_string()
        }
    };

    if !Problem::id_is_legal(&id) {
        return Err(format!("\"{}\" is not a valid problem id", &id).into());
    }

    let kattisrc = cfg.kattisrc()?;
    let host_name = kattisrc.get_host_name()?;
    let url = format!("https://{}/problems/{}", host_name, &id);

    if webbrowser::open(&url).is_err() {
        return Err(format!("failed to open {} in your browser", &url).into());
    }

    println!(
        "{} {} in your browser",
        "opened".bright_green(),
        &url.underline()
    );

    Ok(())
}
