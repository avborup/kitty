use std::path::{Path, PathBuf};
use std::env;
use std::io::{self, Read, Write};
use std::collections::HashMap;
use clap::ArgMatches;
use std::process::{Command, Stdio};
use std::fs::{self, File};
use colored::Colorize;
use crate::lang::Language;
use crate::StdErr;

const CHECKBOX: &'static str = "\u{2705}"; // Green checkbox emoji
const CROSSMARK: &'static str = "\u{274C}"; // Red X emoji

pub async fn test(cmd: &ArgMatches<'_>) -> Result<(), StdErr> {
    // We can unwrap here because clap will exit automatically when this arg is
    // not present.
    let path_arg = cmd.value_of("PATH").unwrap();
    let problem = Problem::from_path(path_arg)?;

    // Find which source file to run. If arg is provided, that takes precedence.
    let file_arg = cmd.value_of("file");
    let file = problem.get_source_file(file_arg)?;
    let file_path_str = file.to_str().expect("file path contains invalid unicode");

    // Find which programming language the solution is written in. If arg is
    // provided, that takes precedence.
    let lang_arg = cmd.value_of("language");
    let lang = get_language_for_file(&file, lang_arg)?;

    match lang {
        Language::Unknown => return Err("kitty doesn't know how to handle this programming language".into()),
        _ => {},
    }

    // Fetch compilation instructions: command to execute and path of executable
    let (compile_cmd, exec_path) = lang.get_compile_instructions(&file_path_str);
    // Unwrapping is fine since we will never add invalid unicode
    let exec_path_str = exec_path.to_str().unwrap();

    // Get the terminal command to run in order to run the source file.
    let run_cmd = if let Some(cmd) = lang.get_run_cmd(exec_path_str) {
        cmd
    } else {
        return Err(format!("kitty doesn't know how to run {} files", lang).into());
    };

    // Collect all pairs of test files from the "test" subfolder (a pair is one
    // .in file and one .ans file)
    let tests = problem.get_test_files()?;

    run_tests(compile_cmd, &run_cmd, &tests)?;

    Ok(())
}

#[derive(Debug)]
struct Problem {
    path: PathBuf,
    name: String,
}

impl Problem {
    fn from_path(path_arg: &str) -> Result<Self, StdErr> {
        let rel_path = Path::new(path_arg).to_path_buf();

        let path = if rel_path.is_absolute() {
            rel_path
        } else {
            let cwd = env::current_dir()?;
            cwd.join(rel_path)
        };

        let path_str = path.to_str().expect("Path did not contain valid unicode");

        if !path.exists() {
             return Err(format!("not found: {}", path_str).into());
        }

        if !path.is_dir() {
            return Err(format!("not a directory: {}", path_str).into());
        }

        let dir = match path.file_name() {
            Some(d) => d,
            None => return Err(format!("failed to get folder name: {}", path_str).into()),
        };

        // We can unwrap because we have already confirmed that the path does
        // not contain invalid unicode
        let name = String::from(dir.to_str().unwrap());

        Ok(Self {
            path,
            name,
        })
    }

    fn path_str(&self) -> &str {
        self.path.to_str().expect("Path did not contain valid unicode")
    }

    fn get_valid_source_files(&self) -> io::Result<Vec<PathBuf>> {
        let entries = self.path.read_dir()?;
        let mut sources = Vec::new();

        for entry in entries {
            let path = entry?.path();

            if path.is_dir() {
                continue;
            }

            let ext = match path.extension() {
                Some(e) => match e.to_str() {
                    Some(e) => e,
                    None => continue,
                },
                None => continue,
            };

            match Language::from_file_ext(ext.to_lowercase().as_str()) {
                Language::Unknown => {},
                _ => sources.push(path),
            };
        }

        Ok(sources)
    }

    fn get_source_file(&self, file_arg: Option<&str>) -> Result<PathBuf, StdErr> {
        let files = self.get_valid_source_files()?;

        if files.len() == 0 {
            return Err(format!("no source files found in {}", self.path_str()).into());
        } else if files.len() > 1 && file_arg.is_none() {
            return Err("multiple source files found - pass the correct source file as an argument".into());
        }

        let file_path = if let Some(file) = file_arg {
            let path = self.path.join(file);

            if !path.exists() {
                let path_str = path.to_str().expect("Path did not contain valid unicode");
                return Err(format!("provided source file not found: {}", path_str).into());
            }

            path
        } else {
            files[0].clone()
        };

        Ok(file_path)
    }

    fn get_test_files(&self) -> Result<Vec<(PathBuf, PathBuf)>, StdErr> {
        let test_path = self.path.join("test");

        if !test_path.exists() {
            return Err(format!(r#"subfolder "test" is missing in {}"#, self.path_str()).into());
        }

        let mut in_files = HashMap::new();
        let mut ans_files = HashMap::new();

        for entry in test_path.read_dir()? {
            let path = entry?.path();

            if !path.is_file() {
                continue;
            }

            let ext = match path.extension() {
                Some(e) => match e.to_str() {
                    Some(e) => e,
                    None => continue,
                },
                None => continue,
            }.to_lowercase();

            // We can unwrap because the file extension check would ensure
            // that the file was skipped if it did not have a valid name
            let name = path.file_stem().unwrap().to_str().unwrap().to_string();

            if ext == "in" {
                in_files.insert(name, path);
            } else if ext == "ans" {
                ans_files.insert(name, path);
            }
        }

        let mut test_files = Vec::new();

        for (in_key, in_file) in in_files.iter() {
            if let Some(ans_file) = ans_files.get(in_key) {
                test_files.push((in_file.clone(), ans_file.clone()))
            }
        }

        test_files.sort_by(|a, b| {
            // We can unwrap for the same reason as before.
            let a_name = a.0.file_stem().unwrap().to_str().unwrap();
            let b_name = b.0.file_stem().unwrap().to_str().unwrap();

            a_name.to_lowercase().cmp(&b_name.to_lowercase())
        });

        Ok(test_files)
    }
}

fn get_language_for_file(file: &PathBuf, lang_arg: Option<&str>) -> Result<Language, StdErr> {
    let ext = match lang_arg {
        Some(e) => e,
        None => match file.extension() {
            Some(e) => e.to_str().expect("Invalid unicode in file extension"),
            None => return Err("file has no file extension".into()),
        },
    };

    let lang = Language::from_file_ext(&ext.to_lowercase());

    match lang {
        Language::Unknown => Err(format!(r#"unknown language "{}""#, ext).into()),
        _ => Ok(lang),
    }
}

fn run_tests(compile_cmd: Option<Vec<&str>>, run_cmd: &[String], tests: &Vec<(PathBuf, PathBuf)>) -> Result<(), StdErr> {
    if let Some(cmd) = compile_cmd {
        let mut compile_parts = cmd.iter();
        // We always define commands ourselves in this source code, so we can
        // guarantee that parts will always have at least one entry.
        let compile_prog = compile_parts.next().unwrap();
        let compile_args: Vec<_> = compile_parts.collect();

        let mut command = Command::new(compile_prog);
        command.args(&compile_args)
            .stderr(Stdio::piped());

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

            println!("{}\n{}\n", "compilation error:".red(), stderr.trim());

            return Err("program failed to compile".into());
        }
    }

    let mut run_parts = run_cmd.iter();
    // We always define commands ourselves in this source code, so we can
    // guarantee that parts will always have at least one entry.
    let run_prog = run_parts.next().unwrap();
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

        let mut command = Command::new(run_prog);
        command.args(&run_args)
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
                Ok(_) => {},
                Err(_) => return Err("failed to write to stdin of program".into()),
            }
        }

        let output = match child.wait_with_output() {
            Ok(o) => o,
            Err(_) => return Err("failed to wait for program output".into()),
        };

        if output.status.success() {
            let stdout = match String::from_utf8(output.stdout) {
                Ok(s) => s,
                Err(_) => return Err("program output (stdout) contained invalid UTF-8".into()),
            };

            let ans_str = ans.trim().replace("\r\n", "\n");
            let out_str = stdout.trim().replace("\r\n", "\n");

            if ans_str == out_str {
                println!("{}", CHECKBOX);
            } else {
                fails += 1;

                println!("{}\n{}\n{}\n\n{}\n{}\n",
                         CROSSMARK, "Expected:".underline(), ans_str, "Actual:".underline(), out_str);
            }
        } else {
            fails += 1;

            let stderr = match String::from_utf8(output.stderr) {
                Ok(s) => s,
                Err(_) => return Err("program output (stderr) contained invalid UTF-8".into()),
            };

            println!("{}\n{}\n", "program error:".red(), stderr.trim());
        }
    }

    let test_result = if fails == 0 { "ok".green() } else { "failed".red() };
    let num_passed = tests.len() - fails;
    println!("\ntest result: {}. {} passed; {} failed.", test_result, num_passed, fails);

    Ok(())
}
