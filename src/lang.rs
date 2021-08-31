use crate::StdErr;
use std::env::consts::EXE_EXTENSION;
use std::fmt;
use std::path::PathBuf;
use Language::*;

#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone)]
pub enum Language {
    Go,
    Haskell,
    Java,
    Python,
    Rust,
    Unknown,
}

impl Language {
    pub fn from_file_ext(ext: &str) -> Self {
        match ext {
            "go" => Go,
            "hs" => Haskell,
            "java" => Java,
            "py" => Python,
            "rs" => Rust,
            _ => Unknown,
        }
    }

    pub fn file_ext(&self) -> &str {
        match self {
            Go => "go",
            Haskell => "hs",
            Java => "java",
            Python => "py",
            Rust => "rs",
            _ => "",
        }
    }

    pub fn from_file(file: &PathBuf) -> Result<Language, StdErr> {
        let ext = match file.extension() {
            Some(e) => e.to_str().expect("invalid unicode in file extension"),
            None => return Err("file has no file extension".into()),
        };

        let lang = Self::from_file_ext(&ext.to_lowercase());

        Ok(lang)
    }

    // FIXME: This function may trust the input path too much.
    pub fn get_compile_instructions(&self, path: &PathBuf) -> (Option<Vec<String>>, PathBuf) {
        let mut dir_path = path.clone();
        dir_path.pop();

        let path_str = path.to_str().expect("path contained invalid unicode");
        let dir_path_str = dir_path.to_str().expect("path contained invalid unicode");

        let cmd = match self {
            Go => None,
            Haskell => Some(vec![
                "ghc",
                "-O2",
                "-ferror-spans",
                "-threaded",
                "-rtsopts",
                path_str,
            ]),
            Java => Some(vec!["javac", path_str]),
            Python => None,
            Rust => Some(vec!["rustc", "--out-dir", dir_path_str, path_str]),
            Unknown => None,
        }
        .map(|v| v.iter().map(|s| s.to_string()).collect::<Vec<String>>());

        let exec_path = match self {
            Go => path.to_owned(),
            Haskell => path.with_extension(EXE_EXTENSION),
            Java => path.with_extension(""),
            Python => path.to_owned(),
            Rust => path.with_extension(EXE_EXTENSION),
            Unknown => PathBuf::new(),
        };

        (cmd, exec_path)
    }

    // FIXME: This function may trust the input path too much.
    pub fn get_run_cmd(&self, file_path: &PathBuf) -> Option<Vec<String>> {
        let mut dir_path = file_path.clone();
        dir_path.pop();

        let cmd = match self {
            Go => vec!["go", "run", file_path.to_str().unwrap()],
            Haskell => vec![file_path.to_str().unwrap()],
            Java => {
                let class_name = file_path.file_stem().unwrap().to_str().unwrap();
                let class_path = dir_path.to_str().unwrap();

                vec!["java", "-cp", class_path, class_name]
            }
            Python => vec!["python", file_path.to_str().unwrap()],
            Rust => vec![file_path.to_str().unwrap()],
            Unknown => return None,
        };

        Some(cmd.iter().map(|s| s.to_string()).collect())
    }

    pub fn has_main_class(&self) -> bool {
        match self {
            Java => true,
            Go | Haskell | Python | Rust | Unknown => false,
        }
    }

    /// Returns an iterator over all `Language` variants except `Unknown`.
    pub fn all() -> impl Iterator<Item = Language> {
        [Go, Haskell, Java, Python, Rust].iter().copied()
    }
}

impl fmt::Display for Language {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let str = match self {
            Go => "Go",
            Haskell => "Haskell",
            Java => "Java",
            Python => "Python 3",
            Rust => "Rust",
            Unknown => "Unknown",
        };

        fmt.write_str(str)?;

        Ok(())
    }
}
