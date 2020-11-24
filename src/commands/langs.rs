use crate::lang::Language;
use crate::StdErr;
use clap::ArgMatches;
use colored::Colorize;

pub async fn langs(_cmd: &ArgMatches<'_>) -> Result<(), StdErr> {
    let mut langs: Vec<(String, String)> = Language::all()
        .map(|l| (l.to_string(), l.file_ext().to_string()))
        .collect();

    langs.sort_by(|a, b| a.cmp(b));

    println!("{:9}  {}", "Name".bright_cyan(), "Extension".bright_cyan());
    for (name, ext) in langs {
        println!("{:9}  {}", name, ext);
    }

    Ok(())
}
