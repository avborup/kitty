use std::path::PathBuf;

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
    Open(OpenArgs),
    Submit(SubmitArgs),
    Test(TestArgs),

    /// List all the languages you can use kitty with based on your config file
    ///
    /// Whenever you need to provide a language as an argument to kitty (for
    /// example --lang when running tests), provide its extension exactly as
    /// shown in the output of this command.
    Langs,

    /// Updates kitty to the latest version
    ///
    /// The currently installed binary will be replaced with the one at
    /// https://github.com/avborup/kitty/releases/latest
    ///
    /// If no binary is found for your architecture and/or operating system,
    /// please open an issue on kitty's GitHub repository! Alternatively, you
    /// can manage the kitty installation yourself via cargo.
    Update,
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

/// Runs a solution through the test cases
///
/// Test cases are located in the 'test' folder within your solution folder.
/// Initially, tests are simply the official test cases from Kattis, but you can
/// add as many test files as you want.
///
/// For each .in (input) file, the file's content is piped into your solution,
/// and your solution's output is compared to the corresponding .ans (answer)
/// file.
///
/// If your solution exits with a non-zero code, it is considered a runtime
/// error.
#[derive(Args, Debug)]
pub struct TestArgs {
    /// The path to the solution folder you want to test
    #[arg(default_value = ".")]
    pub path: PathBuf,

    /// Path to the solution file to test.
    ///
    /// Useful when there are multiple valid files in the solution folder, if
    /// the file doesn't match one of your defined languages, or if the file is
    /// located somewhere else.
    #[arg(short, long)]
    pub file: Option<PathBuf>,

    /// Programming language to use for the solution.
    ///
    /// Useful when the file has another file extension than the one you defined
    /// for the language.
    ///
    /// Write the file extension for the language (java for Java, py for python,
    /// js for JavaScript, etc.).
    #[arg(short, long)]
    pub lang: Option<String>,

    /// If the test folder does not exist, download the test files from Kattis
    #[arg(long, default_value_t = false)]
    pub fetch: bool,

    /// Display how long each test case takes to execute.
    #[arg(short, long, default_value_t = false)]
    pub time: bool,

    /// Re-runs tests every time the source file changes.
    #[arg(short, long, default_value_t = false)]
    pub watch: bool,

    /// Filter which tests to run. Filters are written using regular
    /// expressions to match on test names.
    ///
    /// This can be useful for focusing on a few specific test cases when
    /// debugging.
    ///
    /// Here are some examples:
    /// Only tests containing "custom": --filter custom.
    /// Only the exact test named "custom01": --filter '^custom01$'.
    /// Only tests that are named one of the numbers from 1 to 5: --filter '^[1-5]$'.
    ///
    /// Note that (depending on your shell), you may have to use quotes around
    /// the filter since the shell may try to expand the regular expression as a
    /// glob pattern.
    #[arg(short = 'F', long)]
    pub filter: Option<String>,
}

/// Opens a problem in the browser
#[derive(Args, Debug)]
pub struct OpenArgs {
    /// The ID of the problem as seen in its URL. Defaults to the name of the
    /// current directory.
    pub problem_id: Option<String>,
}

/// Submits a solution to Kattis
#[derive(Args, Debug)]
pub struct SubmitArgs {
    /// The path to the solution folder you want to submit.
    ///
    /// The name of the folder is used as the problem ID when submitting to
    /// Kattis.
    #[arg(default_value = ".")]
    pub path: PathBuf,

    /// Path to the solution file to submit.
    ///
    /// Useful when there are multiple valid files in the solution folder, if
    /// the file doesn't match one of your defined languages, or if the file is
    /// located somewhere else.
    #[arg(short, long)]
    pub file: Option<PathBuf>,

    /// Programming language to use for the solution.
    ///
    /// Useful when the file has another file extension than the one you defined
    /// for the language.
    ///
    /// Write the file extension for the language (java for Java, py for python,
    /// js for JavaScript, etc.).
    #[arg(short, long)]
    pub lang: Option<String>,

    /// Bypass the confirmation prompt by saying yes in advance.
    #[arg(short, long, default_value_t = false)]
    pub yes: bool,

    /// Open the submission on Kattis in your browser.
    #[arg(short, long, default_value_t = false)]
    pub open: bool,
}
