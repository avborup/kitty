use colored::Colorize;
use config::ConfigValues;
use lazy_static::lazy_static;

mod cli;
mod commands;
mod config;
mod kattis_client;
mod lang;
mod problem;
mod utils;

type StdErr = Box<dyn std::error::Error>;

lazy_static! {
    static ref CFG: ConfigValues = {
        let cfg_result = ConfigValues::load();
        exit_if_err(&cfg_result);
        cfg_result.unwrap()
    };
}

#[tokio::main]
async fn main() {
    let app = cli::init();

    let matches = app.get_matches();
    let command_result = match matches.subcommand() {
        ("test", Some(sub)) => commands::test(sub).await,
        ("get", Some(sub)) => commands::get(sub).await,
        ("submit", Some(sub)) => commands::submit(sub).await,
        ("history", Some(sub)) => commands::history(sub).await,
        ("open", Some(sub)) => commands::open(sub).await,
        ("random", Some(sub)) => commands::random(sub).await,
        ("config", Some(sub)) => commands::config(sub).await,
        ("langs", Some(sub)) => commands::langs(sub).await,
        ("update", Some(sub)) => commands::update(sub).await,
        _ => async { Ok(()) }.await,
    };

    exit_if_err(&command_result);
}

fn exit_if_err<T>(res: &Result<T, StdErr>) {
    if let Err(e) = res {
        eprintln!("{}: {}", "error".bright_red(), e);
        std::process::exit(1);
    }
}
