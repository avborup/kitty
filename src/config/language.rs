use std::fmt;

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
}

impl fmt::Display for Language {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str(&self.name)
    }
}
