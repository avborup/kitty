use std::{
    io::{self, Write},
    path::Path,
};

pub fn path_to_str(path: &Path) -> String {
    let path_str = path
        .to_str()
        .expect("path contained invalid unicode")
        .to_string();

    #[cfg(windows)]
    let path_str = path_str.replace(r"\", r"\\");

    path_str
}

/// Prompts the user in the terminal with a yes/no question. Returns `true` when
/// the user responds "y", `false` otherwise.
pub fn prompt_bool(question: &str) -> bool {
    print!("{} (y/n): ", question);
    io::stdout().flush().expect("failed to flush stdout");

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("failed to read input");

    input.trim().to_lowercase() == "y"
}
