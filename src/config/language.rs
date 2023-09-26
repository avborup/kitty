use std::{fmt, path::Path};

use eyre::Context;

use crate::config::prepare_cmd;

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

    pub fn get_program_execution_commands(
        &self,
        file_path: impl AsRef<Path>,
    ) -> crate::Result<ExecuteProgramCommands> {
        let file_path = file_path.as_ref();

        Ok(ExecuteProgramCommands {
            run_cmd: self.get_run_cmd(file_path)?,
            compile_cmd: self.get_compile_cmd(file_path)?,
        })
    }

    pub fn get_run_cmd(&self, file_path: impl AsRef<Path>) -> crate::Result<Vec<String>> {
        prepare_cmd(&self.run_cmd, file_path).wrap_err("Failed to parse the run command")
    }

    pub fn get_compile_cmd(
        &self,
        file_path: impl AsRef<Path>,
    ) -> crate::Result<Option<Vec<String>>> {
        self.compile_cmd
            .as_ref()
            .map(|cmd| prepare_cmd(cmd, file_path))
            .transpose()
            .wrap_err("Failed to parse the compile command")
    }
}

impl fmt::Display for Language {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str(&self.name)
    }
}

#[derive(Debug)]
pub struct ExecuteProgramCommands {
    run_cmd: Vec<String>,
    compile_cmd: Option<Vec<String>>,
}

impl ExecuteProgramCommands {
    pub fn run_cmd(&self) -> &[String] {
        &self.run_cmd
    }

    pub fn compile_cmd(&self) -> Option<&[String]> {
        self.compile_cmd.as_deref()
    }
}
