use std::{
    io::{self, Read},
    path::{Path, PathBuf},
    process,
    sync::{RwLock, RwLockReadGuard},
    time::{Duration, Instant},
};

use color_eyre::owo_colors::OwoColorize;
use eyre::{bail, eyre, Context};

use crate::{
    cli::{DebugAnswerArgs, DebugArgs, DebugInputArgs, DebugSubcommand},
    commands::test::{FAILURE, SUCCESS},
    config::language::{ExecuteProgramCommands, Language},
    solution::{get_all_files_with_known_extension, Solution, SolutionOptions},
    test_io::{run_compile_cmd, run_test, run_with_input, TestCaseError, TestCaseIO},
    utils::{resolve_and_get_file_name, RunningAverager, TimedPrinter},
    App,
};

pub async fn debug(app: &App, args: &DebugArgs) -> crate::Result<()> {
    let input_args = args.input_args();
    let solution = Solution::from_folder(
        app,
        &input_args.path,
        SolutionOptions {
            file_path: input_args.file.as_ref(),
            lang: input_args.lang.as_ref(),
        },
    )?;

    eyre::ensure!(
        solution.debug_dir().is_dir(),
        "You don't have a debug folder. Create it at: {}",
        solution.debug_dir().display().underline()
    );

    let solution_exec_cmds = solution
        .lang
        .get_program_execution_commands(&solution.file)?;

    if let Some(compile_cmd) = solution_exec_cmds.compile_cmd() {
        run_compile_cmd(app, compile_cmd)?;
    }

    let generators = GeneratorTestCase::new(app, &solution, args);

    let num_tests = input_args.num_tests;

    let mut printer = TimedPrinter::new(Duration::from_millis(30));
    let mut averager = RunningAverager::new();

    for n in 1..=num_tests {
        printer.flush_print(
            format!("\r{} {n}/{num_tests}... ", "Running test".bright_cyan()),
            n == num_tests,
        )?;

        let result = test_solution_with_generators(app, &solution, &generators);

        if result.is_err() {
            println!("{FAILURE}\n");
        }

        let outcome = result?;
        let execution_time = outcome
            .as_ref()
            .map_or_else(|e| e.execution_time, |v| v.execution_time);

        averager.add_sample(execution_time.as_secs_f64());

        if let Err(failure) = outcome {
            println!("{FAILURE}\n");

            failure.test_case_error.print();

            println!("{}:", "Input".bright_red());
            println!("{}\n", failure.test_case_error.input().trim_end());

            // TODO: Hide output if too large
            // TODO: Write input/output to files
            // TODO: Helpful message.. "you can copy this to your test directory with .in/.ans files"

            return Ok(());
        }
    }

    println!("{SUCCESS}\n");

    let format_time = |secs: Option<f64>| {
        secs.map(|secs| format!("{secs:.2}s"))
            .unwrap_or_else(|| "N/A".to_string())
    };

    println!(
        "{} all {num_tests} test cases. Running times: min {min_time}, max {max_time}, average {avg_time}.",
        "Passed".bright_green(),
        min_time = format_time(averager.min()),
        max_time = format_time(averager.max()),
        avg_time = format_time(averager.average()),
    );

    Ok(())
}

fn test_solution_with_generators(
    app: &App,
    solution: &Solution,
    test_case: &GeneratorTestCase<'_>,
) -> crate::Result<Result<GeneratorSuccess, GeneratorError>> {
    let start_time = Instant::now();
    let test_result = run_test(app, &solution.lang.get_run_cmd(&solution.file)?, test_case)?;
    let execution_time = start_time.elapsed();

    if let Err(test_case_error) = test_result {
        let should_return_error = match test_case_error {
            TestCaseError::RuntimeError { .. } => true,
            TestCaseError::WrongAnswer { .. } if test_case.should_check_answer() => true,
            _ => false,
        };

        if should_return_error {
            return Ok(Err(GeneratorError {
                test_case_error,
                execution_time,
            }));
        }
    }

    Ok(Ok(GeneratorSuccess { execution_time }))
}

type LazilyInitialised<T> = RwLock<Option<T>>;

struct GeneratorTestCase<'a> {
    app: &'a App,
    solution: &'a Solution<'a>,
    input_args: &'a DebugInputArgs,
    answer_args: Option<&'a DebugAnswerArgs>,
    // Use read-write locks to achieve interior mutability, which we use to only
    // perform initialisation once (i.e. file search and compilation).
    input_generator_exec_cmds: LazilyInitialised<ExecuteProgramCommands>,
    answer_generator_exec_cmds: LazilyInitialised<Option<ExecuteProgramCommands>>,
}

impl TestCaseIO for GeneratorTestCase<'_> {
    type Input<'a> = io::Cursor<Vec<u8>> where Self: 'a;
    type Answer<'a> = io::Cursor<Vec<u8>> where Self: 'a;

    fn input(&self) -> crate::Result<Self::Input<'_>> {
        if !self.has_initialised_input_generator()? {
            let mut initialiser_guard = self
                .input_generator_exec_cmds
                .write()
                .map_err(|_| eyre::eyre!("Failed to lock input generator"))?;

            let (input_generator_path, input_generator_lang) = resolve_generator_file_to_use(
                self.app,
                "input",
                self.solution.debug_dir(),
                self.input_args.input_generator_path.as_ref(),
            )?;

            let exec_cmds =
                input_generator_lang.get_program_execution_commands(input_generator_path)?;

            if let Some(compile_cmd) = exec_cmds.compile_cmd() {
                run_compile_cmd(self.app, compile_cmd)
                    .wrap_err("Failed to compile your input generator")?;
            }

            *initialiser_guard = Some(exec_cmds);
        }

        let exec_cmds_guard = self.get_input_generator_exec_cmds()?;
        let exec_cmds = exec_cmds_guard.as_ref().ok_or_else(|| {
            eyre!("Failed to get input generator run command. This is a bug, please report it!")
        })?;

        let input_generator_output =
            run_with_input(self.app, exec_cmds.run_cmd(), &mut io::empty())?;

        fail_if_output_is_not_success("input", &input_generator_output)?;

        Ok(io::Cursor::new(input_generator_output.stdout))
    }

    fn answer<R: Read>(&self, input: Option<R>) -> crate::Result<Self::Answer<'_>> {
        let mut input: Box<dyn Read> = match input {
            Some(input) => Box::new(input),
            None => Box::new(io::empty()),
        };

        if !self.has_initialised_answer_generator()? {
            let mut initialiser_guard = self
                .answer_generator_exec_cmds
                .write()
                .map_err(|_| eyre::eyre!("Failed to lock answer generator"))?;

            let exec_cmds = match self.answer_args {
                None => None,
                Some(answer_args) => {
                    let (answer_validator_path, answer_validator_lang) =
                        resolve_generator_file_to_use(
                            self.app,
                            "answer",
                            self.solution.debug_dir(),
                            answer_args.answer_validator_path.as_ref(),
                        )?;

                    let exec_cmds = answer_validator_lang
                        .get_program_execution_commands(answer_validator_path)?;

                    if let Some(compile_cmd) = exec_cmds.compile_cmd() {
                        run_compile_cmd(self.app, compile_cmd)
                            .wrap_err("Failed to compile your answer validator")?;
                    }

                    Some(exec_cmds)
                }
            };

            *initialiser_guard = Some(exec_cmds);
        }

        let exec_cmds_guard = self.get_answer_generator_exec_cmds()?;
        let given_exec_cmds = exec_cmds_guard.as_ref().ok_or_else(|| {
            eyre!("Failed to get answer validator run command. This is a bug, please report it!")
        })?;

        let exec_cmds = match given_exec_cmds {
            Some(exec_cmds) => exec_cmds,
            None => return Ok(io::Cursor::new(Vec::new())),
        };

        let answer_generator_output = run_with_input(self.app, exec_cmds.run_cmd(), &mut input)?;

        fail_if_output_is_not_success("answer", &answer_generator_output)?;

        Ok(io::Cursor::new(answer_generator_output.stdout))
    }
}

impl<'a> GeneratorTestCase<'a> {
    pub fn new(app: &'a App, solution: &'a Solution, args: &'a DebugArgs) -> Self {
        Self {
            app,
            solution,
            input_args: args.input_args(),
            answer_args: args.answer_args(),
            input_generator_exec_cmds: RwLock::new(None),
            answer_generator_exec_cmds: RwLock::new(None),
        }
    }

    fn should_check_answer(&self) -> bool {
        self.answer_args.is_some()
    }

    fn get_input_generator_exec_cmds(
        &self,
    ) -> crate::Result<RwLockReadGuard<'_, Option<ExecuteProgramCommands>>> {
        self.input_generator_exec_cmds
            .read()
            .map_err(|_| eyre!("Failed to lock input generator"))
    }

    fn has_initialised_input_generator(&self) -> crate::Result<bool> {
        self.get_input_generator_exec_cmds()
            .map(|cmd| cmd.is_some())
    }

    fn get_answer_generator_exec_cmds(
        &self,
    ) -> crate::Result<RwLockReadGuard<'_, Option<Option<ExecuteProgramCommands>>>> {
        self.answer_generator_exec_cmds
            .read()
            .map_err(|_| eyre!("Failed to lock answer generator"))
    }

    fn has_initialised_answer_generator(&self) -> crate::Result<bool> {
        self.get_answer_generator_exec_cmds()
            .map(|cmd| cmd.is_some())
    }
}

fn fail_if_output_is_not_success(name: &str, output: &process::Output) -> crate::Result<()> {
    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);

    bail!(
        indoc::indoc! {"
            Your {name} generator exited with a non-zero exit code ({status}).

            {label}:
            {stdout}
            {stderr}\
        "},
        label = "Generator output".bright_red(),
        name = name,
        status = output.status,
        stdout = stdout.trim_end(),
        stderr = stderr.trim_end()
    );
}

#[derive(Debug)]
struct GeneratorError {
    test_case_error: TestCaseError,
    execution_time: Duration,
}

#[derive(Debug)]
struct GeneratorSuccess {
    execution_time: Duration,
}

fn resolve_generator_file_to_use(
    app: &App,
    name: impl AsRef<str>,
    debug_dir: impl AsRef<Path>,
    file_path: Option<impl AsRef<Path>>,
) -> crate::Result<(PathBuf, &Language)> {
    let name = name.as_ref();

    if let Some(file_path) = file_path {
        let file_path = file_path.as_ref();

        eyre::ensure!(
            file_path.is_file(),
            "The {name} generator file path does not point to a file: {}",
            file_path.display().underline()
        );

        return Ok((
            file_path.to_path_buf(),
            app.config.try_lang_from_file(file_path)?,
        ));
    }

    let debug_dir = debug_dir.as_ref();
    let options = get_all_files_with_known_extension(app, debug_dir)?
        .into_iter()
        .filter(|path| {
            resolve_and_get_file_name(path)
                .map(|file_name| file_name.contains(name))
                .unwrap_or(false)
        })
        .collect::<Vec<_>>();

    eyre::ensure!(
        !options.is_empty(),
        "No {name} generator file found in the debug folder: {}. See the help message for how to create one.",
        debug_dir.display().underline()
    );

    if let [file] = options.as_slice() {
        return Ok((file.clone(), app.config.try_lang_from_file(file)?));
    }

    eyre::bail!("Multiple {name} generator files found. Specify which file to use.");
}

impl DebugArgs {
    pub fn input_args(&self) -> &DebugInputArgs {
        match &self.subcommand {
            DebugSubcommand::Input(input_args) => input_args,
            DebugSubcommand::Answer(answer_args) => &answer_args.input_args,
        }
    }

    pub fn answer_args(&self) -> Option<&DebugAnswerArgs> {
        match &self.subcommand {
            DebugSubcommand::Input(_) => None,
            DebugSubcommand::Answer(answer_args) => Some(answer_args),
        }
    }
}
