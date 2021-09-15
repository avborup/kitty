use clap::{crate_authors, crate_version, App, AppSettings, Arg, SubCommand};

pub fn init() -> App<'static, 'static> {
    App::new("kitty")
        .version(crate_version!())
        .author(crate_authors!())
        .about("A tool for interacting with Kattis via the command line")
        .setting(AppSettings::GlobalVersion)
        .setting(AppSettings::SubcommandRequired)
        .subcommand(SubCommand::with_name("test")
                    .about("Runs a solution through the official test cases")
                    .setting(AppSettings::DisableVersion)
                    .arg(Arg::with_name("PATH")
                         .help("Path to problem directory")
                         .default_value(".")
                         .index(1))
                    .arg(Arg::with_name("file")
                         .short("f")
                         .long("file")
                         .takes_value(true)
                         .help("Name of source file to test. Necessary when there are multiple valid sources or when the program cannot recognise the file extension"))
                    .arg(Arg::with_name("language")
                         .short("l")
                         .long("lang")
                         .takes_value(true)
                         .help("Programming language of the solution. Write the typical file extension for the language (java for Java, py for python, js for JavaScript, etc.)."))
                    .arg(Arg::with_name("time")
                         .short("t")
                         .long("time")
                         .help("Display how long each test case took to execute"))
                    .arg(Arg::with_name("fetch")
                         .long("fetch")
                         .help("If the test folder does not exist, download the test files from Kattis"))
                    .arg(Arg::with_name("watch")
                         .short("w")
                         .long("watch")
                         .help("Re-runs tests every time the source file changes"))
                   )
        .subcommand(SubCommand::with_name("get")
                    .about("Fetches a problem from Kattis by creating a directory of the same name and downloading the official test cases")
                    .after_help("You can create your own templates for your preferred programming languages. In kitty's config directory, create a \"templates\" subfolder, and inside that, create a file such as template.java in which you define your Java template.")
                    .setting(AppSettings::DisableVersion)
                    .arg(Arg::with_name("PROBLEM ID")
                         .help("ID of the Kattis problem. You can find the id in the URL of the problem page on kattis: open.kattis.com/problems/<PROBLEM ID>")
                         .required(true)
                         .index(1))
                    .arg(Arg::with_name("language")
                         .short("l")
                         .long("lang")
                         .takes_value(true)
                         .help("Programming language to use the template for. Write the typical file extension for the language (java for Java, py for python, js for JavaScript, etc.)."))
                   )
        .subcommand(SubCommand::with_name("submit")
                    .about("Submits a solution to Kattis")
                    .setting(AppSettings::DisableVersion)
                    .arg(Arg::with_name("PATH")
                         .help("Path to problem directory. Note that the directory name must match the problem id")
                         .default_value(".")
                         .index(1))
                    .arg(Arg::with_name("file")
                         .short("f")
                         .long("file")
                         .takes_value(true)
                         .help("Name of source file to test. Necessary when there are multiple valid sources or when the program cannot recognise the file extension"))
                    .arg(Arg::with_name("language")
                         .short("l")
                         .long("lang")
                         .takes_value(true)
                         .help("Programming language of the solution. Write the typical file extension for the language (java for Java, py for python, js for JavaScript, etc.)."))
                    .arg(Arg::with_name("yes")
                         .short("y")
                         .long("yes")
                         .help("Bypass the confirmation prompt by saying \"yes\" in advance"))
                   )
        .subcommand(SubCommand::with_name("history")
                    .about("Shows a list of your submissions to Kattis as seen on your profile page")
                    .setting(AppSettings::DisableVersion)
                    .arg(Arg::with_name("count")
                         .short("c")
                         .long("count")
                         .help("How many submissions to show")
                         .default_value("10"))
                    .arg(Arg::with_name("all")
                         .short("a")
                         .long("all")
                         .help("Show all submissions (if --count is present too, it is ignored)"))
                   )
        .subcommand(SubCommand::with_name("open")
                    .about("Opens a problem in the browser")
                    .setting(AppSettings::DisableVersion)
                    .arg(Arg::with_name("PROBLEM ID")
                         .help("The ID of the problem as seen in its URL. [default: the name of the current directory]")
                         .index(1))
                   )
        .subcommand(SubCommand::with_name("random")
                    .about("Randomly selects an untried problem from Kattis")
                    .after_help("Note: Kattis displays problems 100 at a time as you can see by going to https://open.kattis.com/problems. That also means kitty cannot select a problem at random among every single Kattis problem - instead, kitty selects a random problem among the 100 shown ones. You can specify which 100 to show by passing the sorting options.")
                    .setting(AppSettings::DisableVersion)
                    .arg(Arg::with_name("sort")
                         .short("s")
                         .long("sort-by")
                         .help("Choose which attribute to sort by")
                         .possible_values(&["name", "total", "acc", "ratio", "fastest", "difficulty"])
                         .default_value("name"))
                    .arg(Arg::with_name("direction")
                         .short("d")
                         .long("direction")
                         .help("Direction to sort")
                         .possible_values(&["asc", "desc"])
                         .default_value("asc"))
                    .arg(Arg::with_name("yes")
                         .short("y")
                         .long("yes")
                         .help("Skip the prompt asking if you want to fetch the problem, saying \"yes\" in advance"))
                    .arg(Arg::with_name("language")
                         .short("l")
                         .long("lang")
                         .takes_value(true)
                         .help("Same as the language argument for the get command"))
                   )
        .subcommand(SubCommand::with_name("config")
                    .about("Configures kitty")
                    .setting(AppSettings::DisableVersion)
                    .arg(Arg::with_name("init")
                         .long("init")
                         .help("Creates the kitty config directory in the correct location and shows where it is. If this is provided, all other arguments are ignored"))
                    .arg(Arg::with_name("location")
                         .long("location")
                         .help("Shows where the kitty config directory is or should be located"))
                   )
        .subcommand(SubCommand::with_name("langs")
                    .about("Displays all the languages supported by kitty")
                    .setting(AppSettings::DisableVersion)
                    .after_help("Whenever you need to provide a language as an argument to kitty (for example --lang when fetching commands), provide its extension exactly as shown in the output of this command.")
                   )
        .subcommand(SubCommand::with_name("update")
                    .about("Updates the local installation of kitty")
                    .setting(AppSettings::DisableVersion)
                    .after_help("The currently installed binary will be replaced with the one at https://github.com/KongBorup/kitty/releases/latest")
                   )
}
