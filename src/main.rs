use colored::Colorize;

mod cli;
mod commands;
mod problem;
mod lang;
mod config;
mod kattis_client;

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
