pub mod cli;

struct App {
    args: cli::KittyArgs,
}

pub fn run(args: cli::KittyArgs) -> eyre::Result<()> {
    let app = App { args };

    dbg!(app.args);

    Ok(())
}
