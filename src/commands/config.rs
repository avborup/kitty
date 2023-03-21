use std::fs;

use colored::Colorize;
use eyre::Context;

use crate::{
    cli::{ConfigArgs, ConfigSubcommand},
    App,
};

pub async fn config(app: &App, config_args: &ConfigArgs) -> crate::Result<()> {
    match &config_args.subcommand {
        ConfigSubcommand::Init => {
            init_config_files(app).wrap_err("Failed to initialise config files")
        }
        ConfigSubcommand::Location => {
            show_config_location(app).wrap_err("Failed to load config location")
        }
    }
}

fn init_config_files(app: &App) -> crate::Result<()> {
    fs::create_dir_all(app.config.templates_dir_path())?;

    println!(
        indoc::indoc! {"
            Initialised config directory at {}.
            You should place your .kattisrc and kitty.yml here."
        },
        app.config.config_dir.display().to_string().underline()
    );

    Ok(())
}

fn show_config_location(app: &App) -> crate::Result<()> {
    println!(
        indoc::indoc! {"
            Your config files should go in this directory:

                {}

            More specifically:
             - Your .kattisrc file:   {}
             - Your kitty.yml file:   {}
             - Your templates folder: {}
        "},
        app.config.config_dir.display().to_string().underline(),
        app.config.kattisrc_path().display(),
        app.config.config_file_path().display(),
        app.config.templates_dir_path().display()
    );

    Ok(())
}
