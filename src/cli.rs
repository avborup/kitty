use clap::{Parser, Subcommand};

pub fn parse_args() -> KittyArgs {
    KittyArgs::parse()
}

/// A tool for interacting with Kattis via the command line
#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct KittyArgs {
    /// Enables verbose output
    #[arg(short, long, default_value_t = false, global = true)]
    pub verbose: bool,

    #[command(subcommand)]
    pub subcommand: KittySubcommand,
}

#[derive(Subcommand, Debug)]
pub enum KittySubcommand {
    /// List all the languages you can use kitty with based on your config file
    ///
    /// Whenever you need to provide a language as an argument to kitty (for
    /// example --lang when running tests), provide its extension exactly as
    /// shown in the output of this command.
    Langs,
}
