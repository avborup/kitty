use std::fmt;
use std::path::{Path, PathBuf};
use Language::*;

#[derive(PartialEq, Eq, Hash, Debug)]
pub enum Language {
    Java,
    Python,
    Unknown,
}

impl Language {
    pub fn from_file_ext(ext: &str) -> Self {
        match ext {
            "java" => Java,
            "py" => Python,
            _ => Unknown,
        }
    }

    // FIXME: This function may trust the input path too much.
    pub fn get_compile_instructions<'a>(&self, path: &'a str) -> (Option<Vec<&'a str>>, PathBuf) {
        let cmd = match self {
            Java => Some(vec!["javac", path]),
            Python => None,
            Unknown => None,
        };

        let path = Path::new(path).to_path_buf();
        let exec_path = match self {
            Java => path.with_extension(""),
            Python => path,
            Unknown => PathBuf::new(),
        };

        (cmd, exec_path)
    }

    // FIXME: This function may trust the input path too much.
    pub fn get_run_cmd<'a>(&self, path: &'a str) -> Option<Vec<String>> {
        let file_path = Path::new(path).to_path_buf();
        let mut dir_path = file_path.clone();
        dir_path.pop();

        let cmd = match self {
            Java => {
                let class_name = file_path.file_stem().unwrap().to_str().unwrap();
                let class_path = dir_path.to_str().unwrap();

                vec!["java", "-cp", class_path, class_name]
            },
            Python => vec!["python", file_path.to_str().unwrap()],
            Unknown => return None,
        };

        Some(cmd.iter().map(|s| s.to_string()).collect())
    }
}

impl fmt::Display for Language {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let str = match self {
            Java => "Java",
            Python => "Python",
            Unknown => "Unknown",
        };

        fmt.write_str(str)?;

        Ok(())
    }
}
