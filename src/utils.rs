use std::{
    env,
    io::{self, Write},
    path::{Path, PathBuf},
};

use eyre::Context;

pub fn get_full_path(path: impl AsRef<Path>) -> crate::Result<PathBuf> {
    let path = path.as_ref();

    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        Ok(env::current_dir()?.join(path))
    }
}

pub fn resolve_and_get_file_name(path: impl AsRef<Path>) -> crate::Result<String> {
    let file_name = path
        .as_ref()
        .canonicalize()?
        .file_name()
        .ok_or_else(|| eyre::eyre!("Could not read file name"))?
        .to_str()
        .ok_or_else(|| eyre::eyre!("Could not convert file name to string"))?
        .to_string();

    Ok(file_name)
}

/// Prompts the user in the terminal with a yes/no question. Returns `true` when
/// the user responds "y", `false` otherwise.
pub fn prompt_bool(question: &str) -> crate::Result<bool> {
    print!("{} (y/n): ", question);
    io::stdout().flush().wrap_err("failed to flush stdout")?;

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .wrap_err("failed to read input")?;

    Ok(input.trim().to_lowercase() == "y")
}
