use clap::Parser;

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
}
