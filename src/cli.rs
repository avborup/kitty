use clap::{Args, Parser, Subcommand};

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
    Config(ConfigArgs),

    Get(GetArgs),

    /// List all the languages you can use kitty with based on your config file
    ///
    /// Whenever you need to provide a language as an argument to kitty (for
    /// example --lang when running tests), provide its extension exactly as
    /// shown in the output of this command.
    Langs,
}

/// A utility to help you configure kitty
///
/// Kitty needs a few files to work properly. These include:
///
///  - A .kattisrc file, which you can download from https://open.kattis.com/download/kattisrc.
///    This file is used to authenticate you with Kattis, and it controls which
///    URLs kitty uses to interact with Kattis.
///
///  - A kitty.yml file, which is used to configure kitty's behaviour. This is
///    where you can configure which programming languages you can use, which
///    commands are run, and similar.
///
///  - A templates directory, which contains templates that kitty can auto-copy
///    when you fetch new problems.
#[derive(Args, Debug)]
pub struct ConfigArgs {
    #[command(subcommand)]
    pub subcommand: ConfigSubcommand,
}

#[derive(Subcommand, Debug)]
pub enum ConfigSubcommand {
    /// Creates the kitty config directory for you in the correct location and
    /// shows where it is.
    Init,

    /// Shows where the kitty config directory is or should be located
    Location,
}

/// Fetches a problem from Kattis by creating a solution folder of the same name
/// and downloading the official test cases. If you have defined a template, it
/// will be copied into your solution folder.
///
/// You can create your own templates for your preferred programming languages.
/// In kitty's config directory, create a 'templates' subfolder, and inside that,
/// create a file such as template.java in which you define your Java template.
#[derive(Args, Debug)]
pub struct GetArgs {
    /// The ID of the problem to fetch from Kattis.
    ///
    /// You can find the id in the URL of the problem page on kattis:
    /// open.kattis.com/problems/<PROBLEM ID>
    pub problem_id: String,

    /// If present, remove the host name from the problem ID in templates.
    ///
    /// If the problem ID has a host prefix, this flag will remove it when
    /// inserting the ID into a template. For example, 'itu.flights' becomes
    /// just 'flights'.
    #[arg(short, long, default_value_t = false)]
    pub no_domain: bool,

    /// Programming language to use the template for.
    ///
    /// Write the file extension for the language (java for Java, py for python,
    /// js for JavaScript, etc.).
    #[arg(short, long)]
    pub lang: Option<String>,
}
