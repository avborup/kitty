pub mod cli;

pub type Result<T> = eyre::Result<T>;

struct App {
    args: cli::KittyArgs,
}

pub fn run(args: cli::KittyArgs) -> crate::Result<()> {
    let app = App { args };

    dbg!(app.args);

    Ok(())
}
