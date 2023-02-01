pub mod cli;
mod config;

pub type Result<T> = eyre::Result<T>;

#[derive(Debug)]
struct App {
    args: cli::KittyArgs,
    config: config::Config,
}

pub fn run(args: cli::KittyArgs) -> crate::Result<()> {
    let config = config::Config::load()?;

    let app = App { args, config };

    dbg!(app);

    Ok(())
}
