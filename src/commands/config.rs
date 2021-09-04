use crate::config::Config;
use crate::StdErr;
use clap::ArgMatches;
use colored::Colorize;

pub async fn config(cmd: &ArgMatches<'_>) -> Result<(), StdErr> {
    if cmd.is_present("init") {
        match Config::init() {
            Ok(p) => {
                let path = p.to_str().expect("path contained invalid unicode");
                println!(
                    "{} config directory at {}. you should place your .kattisrc and kitty.yml here",
                    "initialised".bright_green(),
                    path
                );
            }
            Err(_) => return Err("failed to initialise config directory".into()),
        }

        return Ok(());
    }

    if cmd.is_present("location") {
        let dir_path = Config::dir_path();
        let path_str = dir_path.to_str().expect("path contained invalid unicode");

        println!("config directory path: {}", path_str.underline());
    }

    Ok(())
}
