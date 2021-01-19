use colored::Colorize;

mod cli;
mod commands;
mod config;
mod kattis_client;
mod lang;
mod problem;

type StdErr = Box<dyn std::error::Error>;

#[tokio::main]
async fn main() {
    let app = cli::init();

    let matches = app.get_matches();
    let res = match matches.subcommand() {
        ("test", Some(sub)) => commands::test(sub).await,
        ("get", Some(sub)) => commands::get(sub).await,
        ("submit", Some(sub)) => commands::submit(sub).await,
        ("history", Some(sub)) => commands::history(sub).await,
        ("open", Some(sub)) => commands::open(sub).await,
        ("random", Some(sub)) => commands::random(sub).await,
        ("config", Some(sub)) => commands::config(sub).await,
        ("langs", Some(sub)) => commands::langs(sub).await,
        _ => async { Ok(()) }.await,
    };

    std::process::exit(match res {
        Ok(_) => 0,
        Err(e) => {
            eprintln!("{}: {}", "error".bright_red(), e);
            1
        }
    });
}
