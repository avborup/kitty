use std::{
    io::{stdout, Write},
    path::Path,
};

use colored::Colorize;
use eyre::Context;
use notify::{event::ModifyKind, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use regex::Regex;

use crate::{
    cli::TestArgs,
    config::language::ExecuteProgramCommands,
    solution::{get_test_cases, get_test_dir, Solution, SolutionOptions},
    test_io::{run_compile_cmd, run_test},
    utils::prompt_bool,
    App,
};

pub const SUCCESS: &str = "✅";
pub const FAILURE: &str = "❌";

pub async fn test(app: &App, args: &TestArgs) -> crate::Result<()> {
    let solution = Solution::from_folder(
        app,
        &args.path,
        SolutionOptions {
            file_path: args.file.as_ref(),
            lang: args.lang.as_ref(),
        },
    )?;

    fetch_tests_if_needed(app, args, &solution.id, &solution.dir).await?;

    let test_runner = || -> crate::Result<()> {
        let execution_commands = solution
            .lang
            .get_program_execution_commands(&solution.file)?;

        run_tests(app, args, &solution, execution_commands)
    };

    if args.watch {
        watch(&solution, Box::new(test_runner))?;
    } else {
        test_runner()?;
    }

    Ok(())
}

fn run_tests(
    app: &App,
    args: &TestArgs,
    solution: &Solution,
    execution_commands: ExecuteProgramCommands,
) -> crate::Result<()> {
    if let Some(compile_cmd) = execution_commands.compile_cmd() {
        run_compile_cmd(app, compile_cmd)?;
    }

    let mut test_cases = get_test_cases(&solution.dir)?;

    if let Some(filter) = &args.filter {
        let regex_filter = Regex::new(filter)
            .wrap_err("The given test case filter was an invalid regular expression")?;

        test_cases.retain(|test_case| regex_filter.is_match(&test_case.name));
    }

    let mut num_failed_tests = 0;

    println!("Running {} tests\n", test_cases.len());

    for test_case in &test_cases {
        print!("test {} ... ", test_case.name);
        stdout().flush().wrap_err("Failed to flush output")?;

        let outcome = run_test(app, execution_commands.run_cmd(), test_case)?;

        print!("{}", if outcome.is_ok() { SUCCESS } else { FAILURE });

        if args.time {
            if let Ok(test_info) = &outcome {
                print!(" in {:.2}s", test_info.running_time.as_secs_f64());
            }
        }

        if outcome.is_err() {
            num_failed_tests += 1;
        }

        println!();

        if let Err(test_case_error) = outcome {
            test_case_error.print();
        }
    }

    let overall_outcome = if num_failed_tests == 0 {
        "ok".bright_green()
    } else {
        "failed".bright_red()
    };
    let num_passed_tests = test_cases.len() - num_failed_tests;

    println!(
        "\nTest result: {overall_outcome}. {num_passed_tests} passed; {num_failed_tests} failed.",
    );

    Ok(())
}

async fn fetch_tests_if_needed(
    app: &App,
    args: &TestArgs,
    problem_id: &str,
    solution_dir: impl AsRef<Path>,
) -> crate::Result<()> {
    let test_dir = get_test_dir(&solution_dir);

    if test_dir.exists() {
        return Ok(());
    }

    if args.fetch || prompt_bool("No test cases found. Do you want to fetch them from Kattis?")? {
        crate::commands::get::fetch_tests(app, solution_dir, problem_id)
            .await
            .wrap_err("Failed to fetch test cases")?;
    }

    Ok(())
}

fn watch(solution: &Solution, test_runner: impl Fn() -> crate::Result<()>) -> crate::Result<()> {
    let test_runner_wrapper = || {
        if let Err(e) = test_runner() {
            eprintln!("{}: {e}", "Error".bright_red());
        }

        println!(
            "\n{} {}...\n",
            "watching".bright_cyan(),
            solution
                .file
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .underline(),
        );
    };

    let (tx, rx) = std::sync::mpsc::channel();

    let mut watcher = RecommendedWatcher::new(tx, notify::Config::default())
        .wrap_err("Failed to create file watcher")?;

    watcher
        .watch(&solution.file, RecursiveMode::NonRecursive)
        .wrap_err("Failed to watch file")?;

    test_runner_wrapper();
    for event in rx {
        let event = event.wrap_err("Something went wrong during file watching")?;

        if let EventKind::Modify(ModifyKind::Data(_)) = event.kind {
            test_runner_wrapper();
        }
    }

    Ok(())
}
