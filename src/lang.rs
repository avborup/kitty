use crate::StdErr;
use std::env::consts::EXE_EXTENSION;
use std::fmt;
use std::path::{Path, PathBuf};
use Language::*;

#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone)]
pub enum Language {
    CSharp,
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
            "cs" => CSharp,
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
            CSharp => "cs",
            Go => "go",
            Haskell => "hs",
            Java => "java",
            Python => "py",
            Rust => "rs",
            _ => "",
        }
    }

    pub fn from_file(file: &Path) -> Result<Language, StdErr> {
        let ext = match file.extension() {
            Some(e) => e.to_str().expect("invalid unicode in file extension"),
            None => return Err("file has no file extension".into()),
        };

        let lang = Self::from_file_ext(&ext.to_lowercase());

        Ok(lang)
    }

    // FIXME: This function may trust the input path too much.
    pub fn get_compile_instructions(&self, path: &Path) -> (Option<Vec<String>>, PathBuf) {
        let mut dir_path = path.to_path_buf();
        dir_path.pop();

        let path_str = path.to_str().expect("path contained invalid unicode");
        let dir_path_str = dir_path.to_str().expect("path contained invalid unicode");

        // FIXME: Ugly solution for C# command - caused by function returning
        // `Vec<String>` and not `Vec<&str>`.
        let c_sharp_compile_cmd = get_c_sharp_compile_command(path);
        let cmd = match self {
            CSharp => Some(c_sharp_compile_cmd.iter().map(|s| s as &str).collect()),
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
            CSharp => path.with_extension(EXE_EXTENSION),
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
    pub fn get_run_cmd(&self, file_path: &Path) -> Option<Vec<String>> {
        let mut dir_path = file_path.to_path_buf();
        dir_path.pop();

        let cmd = match self {
            CSharp => vec![file_path.to_str().unwrap()],
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
            CSharp | Go | Haskell | Python | Rust | Unknown => false,
        }
    }

    /// Returns an iterator over all `Language` variants except `Unknown`.
    pub fn all() -> impl Iterator<Item = Language> {
        [CSharp, Go, Haskell, Java, Python, Rust].iter().copied()
    }
}

impl fmt::Display for Language {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let str = match self {
            CSharp => "C#",
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

fn get_c_sharp_compile_command(src_path: &Path) -> Vec<String> {
    let mut exe_path = src_path.to_path_buf();
    exe_path.set_extension(EXE_EXTENSION);
    let exe_path_str = exe_path.to_str().expect("path contained invalid unicode");
    let src_path_str = src_path.to_str().expect("path contained invalid unicode");

    #[cfg(windows)]
    let (compiler_cmd, arg_symbol) = ("csc", "/");
    #[cfg(unix)]
    let (compiler_cmd, arg_symbol) = ("mcs", "-");

    vec![
        compiler_cmd.to_string(),
        format!("{}out:{}", arg_symbol, exe_path_str),
        src_path_str.to_string(),
    ]
}
