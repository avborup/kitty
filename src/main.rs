use colored::Colorize;
use kitty::cli;

fn main() -> color_eyre::eyre::Result<()> {
    color_eyre::install()?;

    let args = cli::parse_args();
    let verbose_enabled = args.verbose;

    let result = kitty::run(args);

    exit_if_err(result, verbose_enabled);

    Ok(())
}

fn exit_if_err(res: eyre::Result<()>, verbose_enabled: bool) {
    if let Err(e) = res {
        if verbose_enabled {
            eprintln!("{}: {e:?}", "Error".bright_red());
        } else {
            eprintln!("{}: {e}", "Error".bright_red());
            eprintln!();
            eprintln!("Run with --verbose for more information");
        }

        std::process::exit(1);
    }
}
