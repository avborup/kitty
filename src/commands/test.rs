use std::{
    fs::{self, File},
    io::{self, stdout, Write},
    path::Path,
    process::Command,
    time::Instant,
};

use colored::Colorize;
use eyre::{bail, Context};
use notify::{event::ModifyKind, EventKind, RecommendedWatcher, RecursiveMode, Watcher};

use crate::{
    cli::TestArgs,
    config::language::ExecuteProgramCommands,
    solution::{get_test_cases, get_test_dir, Solution, SolutionOptions, TestCase},
    utils::{get_full_path, prompt_bool},
    App,
};

const CHECKBOX: &str = "\u{2705}"; // Green checkbox emoji
const CROSSMARK: &str = "\u{274C}"; // Red X emoji

pub async fn test(app: &App, args: &TestArgs) -> crate::Result<()> {
    let solution_dir =
        get_full_path(&args.path).wrap_err("Failed to make the solution folder path absolute")?;

    let solution = Solution::from_folder(
        app,
        &solution_dir,
        SolutionOptions {
            file_path: args.file.as_ref(),
            lang: args.lang.as_ref(),
        },
    )?;

    fetch_tests_if_needed(app, args, &solution.id, &solution_dir).await?;

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

    let test_cases = get_test_cases(&solution.dir)?;
    let mut num_failed_tests = 0;

    println!("Running {} tests\n", test_cases.len());

    for test_case in &test_cases {
        print!("test {} ... ", test_case.name);
        stdout().flush().wrap_err("Failed to flush output")?;

        let start_time = Instant::now();
        let outcome = run_test(app, execution_commands.run_cmd(), test_case)?;
        let elapsed_time = start_time.elapsed();

        print!("{}", if outcome.is_ok() { CHECKBOX } else { CROSSMARK });

        if args.time {
            print!(" in {:.2}s", elapsed_time.as_secs_f64());
        }

        if outcome.is_err() {
            num_failed_tests += 1;
        }

        println!();

        match outcome {
            Ok(_) => {}
            Err(TestCaseError::WrongAnswer { expected, actual }) => {
                println!("{}", "Expected:".underline());
                println!("{expected}\n");
                println!("{}", "Actual:".underline());
                println!("{actual}\n");
            }
            Err(TestCaseError::RuntimeError { stdout, stderr }) => {
                println!("{}:", "Runtime error".bright_red());
                if !stdout.is_empty() {
                    println!("{stdout}");
                }
                println!("{stderr}\n");
            }
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

type TestCaseResult = Result<(), TestCaseError>;

enum TestCaseError {
    WrongAnswer { expected: String, actual: String },
    RuntimeError { stdout: String, stderr: String },
}

fn run_test(app: &App, run_cmd: &[String], test_case: &TestCase) -> crate::Result<TestCaseResult> {
    let (run_program, run_program_args) = run_cmd
        .split_first()
        .ok_or_else(|| eyre::eyre!("Run command is empty"))?;

    if app.args.verbose {
        eprintln!(
            "Run command:\n\n   {}\n",
            shlex::join(run_cmd.iter().map(String::as_str))
        );
    }

    let expected_answer =
        fs::read_to_string(&test_case.answer_file).wrap_err("Failed to read answer file")?;
    let input_file = File::open(&test_case.input_file).wrap_err("Failed to open input file")?;

    let output = Command::new(run_program)
        .args(run_program_args)
        .stdin(input_file)
        .output()
        .map_err(|err| match err.kind() {
            io::ErrorKind::NotFound => {
                eyre::eyre!("Failed to find the runner program '{}'", run_program)
            }
            _ => eyre::eyre!("Failed to run the runner program: {}", err),
        })?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if !output.status.success() {
        return Ok(Err(TestCaseError::RuntimeError { stdout, stderr }));
    }

    if !are_equal_after_normalisation(&stdout, &expected_answer) {
        return Ok(Err(TestCaseError::WrongAnswer {
            expected: expected_answer.trim_end().to_string(),
            actual: stdout.trim_end().to_string(),
        }));
    }

    Ok(Ok(()))
}

fn run_compile_cmd(app: &App, compile_cmd: &[String]) -> crate::Result<()> {
    let (compiler_program, compiler_args) = compile_cmd
        .split_first()
        .ok_or_else(|| eyre::eyre!("Compile command is empty"))?;

    if app.args.verbose {
        eprintln!(
            "Compiler command:\n\n   {}\n",
            shlex::join(compile_cmd.iter().map(String::as_str))
        );
    }

    let output = Command::new(compiler_program)
        .args(compiler_args)
        .output()
        .map_err(|err| match err.kind() {
            io::ErrorKind::NotFound => {
                eyre::eyre!("Failed to find the compiler program '{}'", compiler_program)
            }
            _ => eyre::eyre!("Failed to run the compiler program: {}", err),
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);

        eprintln!("{}:", "Compilation error".bright_red());
        eprintln!("{stdout}");
        eprintln!("{stderr}");

        bail!("Failed to compile program ({})", output.status);
    }

    Ok(())
}

fn are_equal_after_normalisation(a: &str, b: &str) -> bool {
    fn normalise(s: &str) -> String {
        s.trim_end()
            .lines()
            .map(str::trim_end)
            .collect::<Vec<_>>()
            .join("\n")
    }

    normalise(a) == normalise(b)
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
