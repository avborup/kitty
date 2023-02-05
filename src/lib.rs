use colored::Colorize;

pub mod cli;
mod commands;
mod config;

pub type Result<T> = eyre::Result<T>;

#[derive(Debug)]
pub struct App {
    args: cli::KittyArgs,
    config: config::Config,
}

pub fn run(args: cli::KittyArgs) {
    let verbose_enabled = args.verbose;

    let result = try_run(args);

    exit_if_err(result, verbose_enabled);
}

fn try_run(args: cli::KittyArgs) -> crate::Result<()> {
    use cli::KittySubcommand::*;

    let config = config::Config::load()?;

    let app = App { args, config };

    match &app.args.subcommand {
        Langs => commands::langs(&app),
        Config(args) => commands::config(&app, args),
    }
}

fn exit_if_err(res: crate::Result<()>, verbose_enabled: bool) {
    if let Err(e) = res {
        if verbose_enabled {
            eprintln!("{}: {e:?}", "Error".bright_red());
        } else {
            eprintln!("{}: {e}", "Error".bright_red());
            eprintln!();
            eprintln!("Run with --verbose for more information");
        }

        std::process::exit(1);
    }
}
