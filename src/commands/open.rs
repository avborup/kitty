use std::env;

use colored::Colorize;
use eyre::Context;

use crate::{
    cli::OpenArgs,
    problem::{make_problem_url, problem_id_is_legal},
    utils::resolve_and_get_file_name,
    App,
};

pub async fn open(app: &App, args: &OpenArgs) -> crate::Result<()> {
    let problem_id = match &args.problem_id {
        Some(problem_id) => problem_id.clone(),
        None => env::current_dir()
            .wrap_err("Failed to get current directory")
            .and_then(resolve_and_get_file_name)
            .wrap_err("Failed to get name of current directory")?,
    };

    if !problem_id_is_legal(&problem_id) {
        eyre::bail!("Problem ID '{problem_id}' is not valid");
    }

    let problem_url = make_problem_url(app, &problem_id)?;

    webbrowser::open(&problem_url).wrap_err("Failed to open your browser")?;

    println!(
        "{} {} in your browser",
        "Opened".bright_green(),
        problem_url.underline()
    );

    Ok(())
}
