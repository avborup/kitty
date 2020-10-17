use std::fmt;
use std::path::PathBuf;
use crate::StdErr;
use Language::*;

#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone)]
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

    pub fn from_file(file: &PathBuf) -> Result<Language, StdErr> {
        let ext = match file.extension() {
            Some(e) => e.to_str().expect("invalid unicode in file extension"),
            None => return Err("file has no file extension".into()),
        };

        let lang = Self::from_file_ext(&ext.to_lowercase());

        Ok(lang)
    }

    // FIXME: This function may trust the input path too much.
    pub fn get_compile_instructions<'a>(&self, path: &'a PathBuf) -> (Option<Vec<&'a str>>, PathBuf) {
        let path_str = path.to_str().expect("path contained invalid unicode");

        let cmd = match self {
            Java => Some(vec!["javac", path_str]),
            Python => None,
            Unknown => None,
        };

        let exec_path = match self {
            Java => path.with_extension(""),
            Python => path.to_owned(),
            Unknown => PathBuf::new(),
        };

        (cmd, exec_path)
    }

    // FIXME: This function may trust the input path too much.
    pub fn get_run_cmd(&self, file_path: &PathBuf) -> Option<Vec<String>> {
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

    pub fn has_main_class(&self) -> bool {
        match self {
            Java => true,
            Python | Unknown => false,
        }
    }
}

impl fmt::Display for Language {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let str = match self {
            Java => "Java",
            Python => "Python 3",
            Unknown => "Unknown",
        };

        fmt.write_str(str)?;

        Ok(())
    }
}
