use std::{
    env, fs,
    io::{self, Write},
    path::{Path, PathBuf},
};

use colored::Colorize;
use eyre::{bail, Context};
use reqwest::StatusCode;
use zip::ZipArchive;

use crate::{
    cli::GetArgs,
    config::{language::Language, Config},
    problem::{make_problem_sample_tests_zip_url, make_problem_url, problem_id_is_legal},
    solution::get_test_dir,
    App,
};

pub async fn get(app: &App, args: &GetArgs) -> crate::Result<()> {
    if !problem_id_is_legal(&args.problem_id) {
        bail!("The given problem ID is invalid. It must only contain alphanumeric characters and periods.");
    }

    if !problem_exists(app, &args.problem_id).await? {
        bail!("Problem '{}' does not exist", args.problem_id)
    }

    let solution_dir = create_solution_dir(&args.problem_id)?;

    fetch_tests(app, &solution_dir, &args.problem_id)
        .await
        .wrap_err("Failed to fetch test cases")?;

    populate_template(app, args, &solution_dir)?;

    println!(
        "{} solution folder for {}",
        "created".bright_green(),
        args.problem_id
    );

    Ok(())
}

async fn problem_exists(app: &App, problem_id: &str) -> crate::Result<bool> {
    let url = make_problem_url(app, problem_id)?;
    let response = app
        .client
        .get(&url)
        .send()
        .await
        .wrap_err("Failed to send request to Kattis")?;

    match response.status() {
        status if status.is_success() => Ok(true),
        StatusCode::NOT_FOUND => Ok(false),
        _ => bail!(
            "Failed to get problem from Kattis (http status code: {})",
            response.status()
        ),
    }
}

fn create_solution_dir(problem_id: &str) -> crate::Result<PathBuf> {
    let cwd = env::current_dir().wrap_err("Failed to get current working directory")?;
    let solution_dir = cwd.join(problem_id);

    let result = fs::create_dir(&solution_dir);

    if let Err(e) = &result {
        if let io::ErrorKind::AlreadyExists = e.kind() {
            bail!("Cannot create solution directory since it already exists");
        }

        result.wrap_err("Failed to create solution directory at this location")?;
    }

    Ok(solution_dir)
}

pub async fn fetch_tests(
    app: &App,
    solution_dir: impl AsRef<Path>,
    problem_id: &str,
) -> crate::Result<()> {
    let test_dir = get_test_dir(solution_dir);

    fs::create_dir(&test_dir).wrap_err("Failed to create test files directory")?;

    let zip_url = make_problem_sample_tests_zip_url(app, problem_id)?;
    let zip_response = app
        .client
        .get(&zip_url)
        .send()
        .await
        .wrap_err("Failed to send request to Kattis")?;

    let status = zip_response.status();
    if !status.is_success() {
        if status == StatusCode::NOT_FOUND {
            return Ok(());
        }

        bail!("Failed to fetch tests from Kattis (http status code: {status})",);
    }

    let response_bytes = zip_response.bytes().await?;
    let mut tmpfile = tempfile::tempfile()?;
    tmpfile.write_all(&response_bytes)?;

    let mut zip = ZipArchive::new(tmpfile)?;

    for i in 0..zip.len() {
        let mut file = zip.by_index(i)?;

        eyre::ensure!(!file.is_dir(), "Zip file contained directory");

        let dest_path = test_dir.join(file.name());
        let mut dest = fs::File::create(&dest_path)
            .wrap_err_with(|| format!("Failed to create file at '{}'", dest_path.display()))?;

        io::copy(&mut file, &mut dest)?;
    }

    Ok(())
}

fn populate_template(
    app: &App,
    args: &GetArgs,
    solution_dir: impl AsRef<Path>,
) -> crate::Result<()> {
    let lang =
        match &args.lang {
            Some(lang) => Some(app.config.lang_from_file_ext(lang).ok_or_else(|| {
                eyre::eyre!("Could not find a language to use for .{} files", lang)
            })?),
            None => app.config.default_language(),
        };

    if let Some(language) = lang {
        copy_template_with_lang(args, solution_dir, language)
            .wrap_err("Failed to populate the solution folder with your template")?;
    }

    Ok(())
}

fn copy_template_with_lang(
    args: &GetArgs,
    solution_dir: impl AsRef<Path>,
    lang: &Language,
) -> crate::Result<()> {
    let templates_dir = Config::templates_dir_path();
    let template_file = templates_dir
        .join("template")
        .with_extension(lang.file_ext());

    if !template_file.exists() {
        println!(
            "{} does not exist. kitty will skip creating the solution file for you.",
            template_file.display()
        );
        return Ok(());
    }

    let file_name = if args.no_domain {
        args.problem_id.split('.').last().unwrap()
    } else {
        &args.problem_id
    };

    let template = fs::read_to_string(&template_file)
        .wrap_err_with(|| eyre::eyre!("failed to read {}", template_file.display()))?
        .replace("$FILENAME", file_name);

    let solution_file = solution_dir
        .as_ref()
        .join(file_name)
        .with_extension(lang.file_ext());

    fs::write(&solution_file, template)
        .wrap_err_with(|| eyre::eyre!("failed to write {}", solution_file.display()))?;

    Ok(())
}
