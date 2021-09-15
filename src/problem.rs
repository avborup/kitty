use crate::lang::Language;
use crate::StdErr;
use crate::CFG as cfg;
use clap::ArgMatches;
use regex::Regex;
use std::collections::HashMap;
use std::env;
use std::io;
use std::path::{Path, PathBuf};

fn path_str(p: &Path) -> &str {
    p.to_str().expect("path did not contain valid unicode")
}

#[derive(Debug)]
pub struct Problem<'a> {
    name: String,
    path: PathBuf,
    file: PathBuf,
    lang: &'a Language,
}

impl<'a> Problem<'a> {
    pub fn from_args(cmd: &ArgMatches) -> Result<Self, StdErr> {
        // We can unwrap here because clap will exit automatically when this arg
        // is not present.
        let path_arg = cmd.value_of("PATH").unwrap();
        let path = Self::get_path(path_arg)?;

        let dir = match path.file_name() {
            Some(d) => d,
            None => return Err(format!("failed to get folder name: {}", path_str(&path)).into()),
        };

        // We can unwrap because we have already confirmed that the path does
        // not contain invalid unicode
        let name = String::from(dir.to_str().unwrap());

        // Find which source file to run. If provided as an argument, that takes
        // precedence.
        let file_arg = cmd.value_of("file");
        let file = Self::get_source_file(&path, file_arg)?;

        // Find which programming language the solution is written in. If arg is
        // provided, that takes precedence.
        let lang_arg = cmd.value_of("language");
        let lang = match lang_arg {
            Some(e) => cfg.lang_from_file_ext(e),
            None => cfg.lang_from_file(&file)?,
        };

        match lang {
            Some(l) => Ok(Self {
                name,
                path,
                file,
                lang: l,
            }),
            None => Err(match lang_arg {
                Some(l) => format!("kitty doesn't know how to handle {} files", l),
                None => {
                    "kitty doesn't know the file extension of the given source file".to_string()
                }
            }
            .into()),
        }
    }

    pub fn id_is_legal(id: &str) -> bool {
        Regex::new(r"^[\w\d\.]+$").unwrap().is_match(id)
    }

    pub fn get_path(path_arg: &str) -> Result<PathBuf, StdErr> {
        let rel_path = Path::new(path_arg).to_path_buf();

        let path = if rel_path.is_absolute() {
            rel_path
        } else {
            let cwd = env::current_dir()?;
            cwd.join(rel_path)
        };

        let path_str = path.to_str().expect("path did not contain valid unicode");

        if !path.exists() {
            return Err(format!("not found: {}", path_str).into());
        }

        if !path.is_dir() {
            return Err(format!("not a directory: {}", path_str).into());
        }

        Ok(path)
    }

    fn path_str(&self) -> &str {
        self.path
            .to_str()
            .expect("path did not contain valid unicode")
    }

    fn get_valid_source_files(dir: &Path) -> io::Result<Vec<PathBuf>> {
        let entries = dir.read_dir()?;
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

            match cfg.lang_from_file_ext(ext) {
                None => {}
                _ => sources.push(path),
            };
        }

        Ok(sources)
    }

    pub fn get_source_file(dir: &Path, file_arg: Option<&str>) -> Result<PathBuf, StdErr> {
        let files = Self::get_valid_source_files(dir)?;

        if files.is_empty() {
            return Err(format!("no source files found in {}", path_str(dir)).into());
        } else if files.len() > 1 && file_arg.is_none() {
            return Err(
                "multiple source files found - pass the correct source file as an argument".into(),
            );
        }

        let file_path = if let Some(file) = file_arg {
            let path = dir.join(file);

            if !path.exists() {
                return Err(format!("provided source file not found: {}", path_str(&path)).into());
            }

            path
        } else {
            files[0].clone()
        };

        Ok(file_path)
    }

    /// Collects all pairs of test files from the "test" subfolder (a pair is
    /// one `.in` file and one `.ans` file with the same name)
    pub fn get_test_files(&self) -> Result<Vec<(PathBuf, PathBuf)>, StdErr> {
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
            }
            .to_lowercase();

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

    pub fn lang(&self) -> &Language {
        self.lang
    }

    pub fn file(&self) -> PathBuf {
        self.file.clone()
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }
}
