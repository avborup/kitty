pub mod cli;
mod commands;
mod config;

pub type Result<T> = eyre::Result<T>;

#[derive(Debug)]
pub struct App {
    args: cli::KittyArgs,
    config: config::Config,
}

pub fn run(args: cli::KittyArgs) -> crate::Result<()> {
    use cli::KittySubcommand::*;

    let config = config::Config::load()?;

    let app = App { args, config };

    match &app.args.subcommand {
        Langs => commands::langs(&app),
    }
}
