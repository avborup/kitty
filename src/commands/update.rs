use crate::StdErr;
use clap::ArgMatches;
use colored::Colorize;
use self_update::{cargo_crate_version, Status};

pub async fn update(_cmd: &ArgMatches<'_>) -> Result<(), StdErr> {
    let cur_version = cargo_crate_version!();
    let status_res = self_update::backends::github::Update::configure()
        .repo_owner("KongBorup")
        .repo_name("kitty")
        .bin_name("kitty")
        .current_version(cur_version)
        .build()?
        .update();

    let status = match status_res {
        Ok(s) => s,
        Err(_) => return Err("failed to update kitty".into()),
    };

    match status {
        Status::Updated(v) => println!(
            "{} kitty: v{} -> v{}",
            "updated".bright_green(),
            cur_version,
            v,
        ),
        Status::UpToDate(v) => println!("kitty is already up to date: v{v}"),
    }

    Ok(())
}
