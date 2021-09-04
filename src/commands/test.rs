use crate::problem::Problem;
use crate::StdErr;
use clap::ArgMatches;
use colored::Colorize;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time;

const CHECKBOX: &str = "\u{2705}"; // Green checkbox emoji
const CROSSMARK: &str = "\u{274C}"; // Red X emoji

pub async fn test(cmd: &ArgMatches<'_>) -> Result<(), StdErr> {
    let problem = Problem::from_args(cmd)?;
    let lang = problem.lang();
    let file = problem.file();

    let compile_cmd = lang.get_compile_cmd(&file)?;
    let run_cmd = lang.get_run_cmd(&file)?;
    let tests = problem.get_test_files()?;

    run_tests(compile_cmd, &run_cmd, &tests, cmd)?;

    Ok(())
}

fn run_tests(
    compile_cmd: Option<Vec<String>>,
    run_cmd: &[String],
    tests: &[(PathBuf, PathBuf)],
    cmd: &ArgMatches<'_>,
) -> Result<(), StdErr> {
    if let Some(cmd) = compile_cmd {
        let mut compile_parts = cmd.iter();
        let compile_prog = compile_parts
            .next()
            .ok_or_else::<StdErr, _>(|| "compile command was empty".into())?;
        let compile_args: Vec<_> = compile_parts.collect();

        let mut command = Command::new(compile_prog);
        command.args(&compile_args).stderr(Stdio::piped());

        let child = match command.spawn() {
            Ok(c) => c,
            Err(_) => return Err(format!("failed to execute command \"{}\"", compile_prog).into()),
        };

        let output = match child.wait_with_output() {
            Ok(o) => o,
            Err(_) => return Err("failed to wait for compilation output".into()),
        };

        if !output.status.success() {
            let stderr = match String::from_utf8(output.stderr) {
                Ok(s) => s,
                Err(_) => return Err("compilation output stderr contained invalid UTF-8".into()),
            };

            println!("{}:\n{}\n", "compilation error".bright_red(), stderr.trim());

            return Err("program failed to compile".into());
        }
    }

    let mut run_parts = run_cmd.iter();
    let run_prog = run_parts
        .next()
        .ok_or_else::<StdErr, _>(|| "run command was empty".into())?;
    let run_args: Vec<_> = run_parts.collect();
    let mut fails = 0;

    println!("running {} tests", tests.len());

    for (test_in, test_ans) in tests {
        // We can unwrap because the file extension check earlier would ensure
        // that the file was skipped if it did not have a valid name
        let test_label = test_in.file_stem().unwrap().to_str().unwrap().to_string();

        print!("test {} ... ", test_label);

        let mut f_in = File::open(test_in)?;
        let mut in_buf = Vec::new();
        f_in.read_to_end(&mut in_buf)?;

        let ans = fs::read_to_string(test_ans)?;

        let start_time = time::Instant::now();
        let mut command = Command::new(run_prog);
        command
            .args(&run_args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut child = match command.spawn() {
            Ok(c) => c,
            Err(_) => return Err(format!("failed to execute command \"{}\"", run_prog).into()),
        };

        {
            let child_stdin = match child.stdin.as_mut() {
                Some(c) => c,
                None => return Err("failed to capture stdin of program".into()),
            };

            match child_stdin.write_all(&in_buf) {
                Ok(_) => {}
                Err(_) => return Err("failed to write to stdin of program".into()),
            }
        }

        let output = match child.wait_with_output() {
            Ok(o) => o,
            Err(_) => return Err("failed to wait for program output".into()),
        };
        let elapsed_time = start_time.elapsed();

        let stdout = match String::from_utf8(output.stdout) {
            Ok(s) => s,
            Err(_) => return Err("program output (stdout) contained invalid UTF-8".into()),
        };

        if output.status.success() {
            let ans_str = reformat_ans_str(&ans);
            let out_str = reformat_ans_str(&stdout);
            let is_success = ans_str == out_str;

            print!("{}", if is_success { CHECKBOX } else { CROSSMARK });

            if cmd.is_present("time") {
                print!(" in {:.2}s", elapsed_time.as_secs_f64());
            }

            println!();

            if !is_success {
                fails += 1;

                println!(
                    "{}\n{}\n\n{}\n{}\n",
                    "Expected:".underline(),
                    ans_str,
                    "Actual:".underline(),
                    out_str
                );
            }
        } else {
            fails += 1;

            let stderr = match String::from_utf8(output.stderr) {
                Ok(s) => s,
                Err(_) => return Err("program output (stderr) contained invalid UTF-8".into()),
            };

            println!(
                "{}:\n{}\n{}\n",
                "program error".bright_red(),
                stdout.trim(),
                stderr.trim()
            );
        }
    }

    let test_result = if fails == 0 {
        "ok".bright_green()
    } else {
        "failed".bright_red()
    };
    let num_passed = tests.len() - fails;
    println!(
        "\ntest result: {}. {} passed; {} failed.",
        test_result, num_passed, fails
    );

    Ok(())
}

fn reformat_ans_str(s: &str) -> String {
    s.replace("\r\n", "\n")
        .trim()
        .lines()
        .map(str::trim)
        .collect::<Vec<&str>>()
        .join("\n")
}
