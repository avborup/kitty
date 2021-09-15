use crate::{config, StdErr};
use std::fmt;
use std::path::Path;

#[derive(Debug)]
pub struct Language {
    name: String,
    file_ext: String,
    run_cmd: String,
    compile_cmd: Option<String>,
}

impl Language {
    pub fn new(
        name: String,
        file_ext: String,
        run_cmd: String,
        compile_cmd: Option<String>,
    ) -> Self {
        // TODO: Add some sort of input validation (ex. valid file extension, strip whitespace etc.)
        Language {
            name,
            file_ext,
            run_cmd,
            compile_cmd,
        }
    }

    pub fn file_ext(&self) -> &str {
        &self.file_ext
    }

    pub fn get_run_cmd(&self, file_path: &Path) -> Result<Vec<String>, StdErr> {
        config::prepare_cmd(&self.run_cmd, file_path)
            .ok_or_else(|| "failed to parse run command".into())
    }

    pub fn get_compile_cmd(&self, file_path: &Path) -> Result<Option<Vec<String>>, StdErr> {
        let cmd_str = match &self.compile_cmd {
            Some(c) => c,
            None => return Ok(None),
        };

        match config::prepare_cmd(cmd_str, file_path) {
            Some(c) => Ok(Some(c)),
            None => Err("failed to parse compile command".into()),
        }
    }
}

impl fmt::Display for Language {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str(&self.name)
    }
}
