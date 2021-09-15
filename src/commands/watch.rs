use crate::commands::test;
use crate::{problem::Problem, StdErr};
use clap::ArgMatches;
use colored::Colorize;
use notify::{watcher, DebouncedEvent, RecursiveMode, Watcher};
use std::sync::mpsc::channel;
use std::time::Duration;

pub async fn watch(cmd: &ArgMatches<'_>) -> Result<(), StdErr> {
    let problem = Problem::from_args(cmd)?;
    let src_file = problem.file();

    let (tx, rx) = channel();
    let mut watcher = watcher(tx, Duration::from_secs(1)).unwrap();

    watcher.watch(&src_file, RecursiveMode::NonRecursive)?;

    let print_watching = || {
        println!(
            "{} {}...\n",
            "watching".bright_cyan(),
            src_file
                .file_name()
                .expect("couldn't read file name")
                .to_string_lossy()
                .underline(),
        );
    };

    print_watching();
    loop {
        match rx.recv() {
            Ok(event) => {
                if let DebouncedEvent::NoticeWrite(_) = event {
                    test(cmd).await?;
                    println!();
                    print_watching();
                }
            }
            Err(_) => return Err("something went wrong during file watching".into()),
        }
    }
}
