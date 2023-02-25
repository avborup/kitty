use colored::Colorize;
use eyre::Context;
use self_update::{cargo_crate_version, Status};
use tokio::task::spawn_blocking;

pub async fn update() -> crate::Result<()> {
    let cur_version = cargo_crate_version!();

    // `spawn_blocking` is used to avoid the panic below, which happens because
    // `self_update` uses `reqwest`'s blocking client internally. Calling
    // `reqwest`'s blocking client from an async context causes a panic:
    // https://github.com/seanmonstar/reqwest/issues/1017
    let status = spawn_blocking(move || {
        self_update::backends::github::Update::configure()
            .repo_owner("avborup")
            .repo_name("kitty")
            .bin_name("kitty")
            .current_version(cur_version)
            .build()
            .wrap_err("Failed to construct updater")?
            .update()
            .wrap_err("Failed to update kitty")
    })
    .await??;

    match status {
        Status::Updated(new_version) => {
            println!(
                "{} kitty: v{cur_version} -> v{new_version}",
                "Updated".bright_green(),
            )
        }
        Status::UpToDate(v) => println!("kitty is already up to date: v{v}"),
    }

    Ok(())
}
