use kitty::cli::parse_args;

fn main() -> kitty::Result<()> {
    color_eyre::install()?;

    let args = parse_args();
    kitty::run(args);

    Ok(())
}
