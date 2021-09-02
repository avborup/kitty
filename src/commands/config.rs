use crate::config::Config;
use crate::StdErr;
use crate::CFG as cfg_vals;
use clap::ArgMatches;
use colored::Colorize;

pub async fn config(cmd: &ArgMatches<'_>) -> Result<(), StdErr> {
    if cmd.is_present("init") {
        match Config::init() {
            Ok(p) => {
                let path = p.to_str().expect("path contained invalid unicode");
                println!(
                    "{} config directory at {}. you should place your .kattisrc here",
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

    let mut cfg = Config::load()?;

    if let Some(lang_str) = cmd.value_of("default language") {
        let lang = match cfg_vals.lang_from_file_ext(lang_str) {
            Some(l) => l,
            None => {
                return Err(format!("kitty does not know how to handle .{} files", lang_str).into())
            }
        };

        cfg.set_default_lang(&lang);
        cfg.save()?;

        println!(
            "{} set default language to {}",
            "successfully".bright_green(),
            lang.to_string()
        );
    }

    Ok(())
}
