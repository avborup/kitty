use kitty::cli::parse_args;

#[tokio::main]
async fn main() -> kitty::Result<()> {
    color_eyre::install()?;

    let args = parse_args();
    kitty::run(args).await;

    Ok(())
}
