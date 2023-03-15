use std::{
    fs::File,
    io::{self, stdout, Read, Write},
    path::{Path, PathBuf},
    process::{self, Command, Stdio},
    time::Instant,
};

use colored::Colorize;
use eyre::{bail, Context};
use notify::{event::ModifyKind, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use regex::Regex;

use crate::{
    cli::TestArgs,
    config::language::ExecuteProgramCommands,
    solution::{get_test_cases, get_test_dir, Solution, SolutionOptions},
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

        let start_time = Instant::now();
        let outcome = run_test(app, execution_commands.run_cmd(), test_case)?;
        let elapsed_time = start_time.elapsed();

        print!("{}", if outcome.is_ok() { SUCCESS } else { FAILURE });

        if args.time {
            print!(" in {:.2}s", elapsed_time.as_secs_f64());
        }

        if outcome.is_err() {
            num_failed_tests += 1;
        }

        println!();

        if let Err(test_case_error) = outcome {
            print_test_case_error(&test_case_error);
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

pub fn run_test<'a, T: TestCaseIO + 'a>(
    app: &App,
    run_cmd: &[String],
    test_case: &'a T,
) -> crate::Result<TestCaseResult>
where
    <T as TestCaseIO>::Input<'a>: Read,
    <T as TestCaseIO>::Answer<'a>: Read,
{
    let input = test_case.input()?;
    let input = io::read_to_string(input).wrap_err("Failed to load input")?;

    let expected_answer = test_case.answer(Some(input.as_bytes()))?;
    let expected_answer =
        io::read_to_string(expected_answer).wrap_err("Failed to load expected answer")?;

    let output = run_with_input(app, run_cmd, &mut input.as_bytes())?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if !output.status.success() {
        return Ok(Err(TestCaseError::RuntimeError {
            input,
            stdout,
            stderr,
        }));
    }

    if !are_equal_after_normalisation(&stdout, &expected_answer) {
        return Ok(Err(TestCaseError::WrongAnswer {
            input,
            expected: expected_answer.trim_end().to_string(),
            actual: stdout.trim_end().to_string(),
        }));
    }

    Ok(Ok(()))
}

pub fn print_test_case_error(err: &TestCaseError) {
    match err {
        TestCaseError::WrongAnswer {
            expected, actual, ..
        } => {
            println!("{}", "Expected:".underline());
            println!("{}\n", expected.trim_end());
            println!("{}", "Actual:".underline());
            println!("{}\n", actual.trim_end());
        }
        TestCaseError::RuntimeError { stdout, stderr, .. } => {
            println!("{}:", "Runtime error".bright_red());
            println!("{}", stdout.trim_end());
            println!("{}\n", stderr.trim_end());
        }
    }
}

pub trait TestCaseIO {
    type Input<'a>
    where
        Self: 'a;

    type Answer<'a>
    where
        Self: 'a;

    fn input<'a>(&'a self) -> crate::Result<Self::Input<'a>>;
    fn answer<'a, R>(&'a self, input: Option<R>) -> crate::Result<Self::Answer<'a>>
    where
        R: Read;
}

pub struct FileTestCase {
    pub name: String,
    pub input_file: PathBuf,
    pub answer_file: PathBuf,
}

impl TestCaseIO for FileTestCase {
    type Input<'a> = File;
    type Answer<'a> = File;

    fn input<'a>(&'a self) -> crate::Result<Self::Input<'a>> {
        File::open(&self.input_file).wrap_err("Failed to open input file")
    }

    fn answer<'a, R: Read>(&'a self, _input: Option<R>) -> crate::Result<Self::Answer<'a>> {
        File::open(&self.answer_file).wrap_err("Failed to open answer file")
    }
}

pub type TestCaseResult = Result<(), TestCaseError>;

#[derive(Debug)]
pub enum TestCaseError {
    WrongAnswer {
        input: String,
        expected: String,
        actual: String,
    },
    RuntimeError {
        input: String,
        stdout: String,
        stderr: String,
    },
}

impl TestCaseError {
    pub fn input(&self) -> &str {
        match self {
            TestCaseError::RuntimeError { ref input, .. } => input,
            TestCaseError::WrongAnswer { ref input, .. } => input,
        }
    }
}

pub fn run_with_input(
    app: &App,
    run_cmd: &[String],
    input: &mut impl Read,
) -> crate::Result<process::Output> {
    let (run_program, run_program_args) = run_cmd
        .split_first()
        .ok_or_else(|| eyre::eyre!("Run command is empty"))?;

    if app.args.verbose {
        eprintln!(
            "Run command:\n\n   {}\n",
            shlex::join(run_cmd.iter().map(String::as_str))
        );
    }

    let mut child = Command::new(run_program)
        .args(run_program_args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|err| match err.kind() {
            io::ErrorKind::NotFound => {
                eyre::eyre!("Failed to find the runner program '{}'", run_program)
            }
            _ => eyre::eyre!("Failed to run the runner program: {}", err),
        })?;

    let mut child_stdin = child
        .stdin
        .take()
        .ok_or_else(|| eyre::eyre!("Failed to capture stdin of your solution"))?;

    io::copy(input, &mut child_stdin)
        .wrap_err("Failed to write test case input to your solution")?;

    // Manually drop stdin to ensure that EOF is sent. If this is not done, the
    // child process might not terminate if it reads until EOF.
    drop(child_stdin);

    let output = child
        .wait_with_output()
        .wrap_err("Failed to run the solution")?;

    Ok(output)
}

pub fn run_compile_cmd(app: &App, compile_cmd: &[String]) -> crate::Result<()> {
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
