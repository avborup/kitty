use std::{
    fs::File,
    io::{self, Read},
    path::PathBuf,
    process::{self, Command, Stdio},
};

use colored::Colorize;
use eyre::{bail, Context};

use crate::App;

pub trait TestCaseIO {
    type Input<'a>
    where
        Self: 'a;

    type Answer<'a>
    where
        Self: 'a;

    fn input(&self) -> crate::Result<Self::Input<'_>>;
    fn answer<R>(&self, input: Option<R>) -> crate::Result<Self::Answer<'_>>
    where
        R: Read;
}

pub type TestCaseResult = Result<(), TestCaseError>;

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

pub fn are_equal_after_normalisation(a: &str, b: &str) -> bool {
    fn normalise(s: &str) -> String {
        s.trim_end()
            .lines()
            .map(str::trim_end)
            .collect::<Vec<_>>()
            .join("\n")
    }

    normalise(a) == normalise(b)
}

pub struct FileTestCase {
    pub name: String,
    pub input_file: PathBuf,
    pub answer_file: PathBuf,
}

impl TestCaseIO for FileTestCase {
    type Input<'a> = File;
    type Answer<'a> = File;

    fn input(&self) -> crate::Result<Self::Input<'_>> {
        File::open(&self.input_file).wrap_err("Failed to open input file")
    }

    fn answer<R: Read>(&self, _input: Option<R>) -> crate::Result<Self::Answer<'_>> {
        File::open(&self.answer_file).wrap_err("Failed to open answer file")
    }
}

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

    pub fn print(&self) {
        match self {
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
}
