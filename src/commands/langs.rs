use crate::StdErr;
use crate::CFG as cfg;
use clap::ArgMatches;
use colored::Colorize;

pub async fn langs(_cmd: &ArgMatches<'_>) -> Result<(), StdErr> {
    let mut langs: Vec<(String, &str)> = cfg
        .languages()
        .map(|l| (l.to_string(), l.file_ext()))
        .collect();

    langs.sort();

    println!("{:9}  {}", "Name".bright_cyan(), "Extension".bright_cyan());
    for (name, ext) in langs {
        println!("{name:9}  {ext}");
    }

    Ok(())
}
