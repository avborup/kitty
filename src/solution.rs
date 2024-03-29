use std::{
    collections::BTreeMap,
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
};

use color_eyre::owo_colors::OwoColorize;
use eyre::{Context, ContextCompat};

use crate::{
    config::language::Language,
    test_io::FileTestCase,
    utils::{get_full_path, resolve_and_get_file_name},
    App,
};

#[derive(Debug)]
pub struct Solution<'a> {
    pub id: String,
    pub dir: PathBuf,
    pub file: PathBuf,
    pub lang: &'a Language,
}

impl<'a> Solution<'a> {
    pub fn from_folder(
        app: &'a App,
        path: impl AsRef<Path>,
        options: SolutionOptions,
    ) -> crate::Result<Self> {
        let solution_dir = get_full_path(path)?;

        eyre::ensure!(
            solution_dir.is_dir(),
            "The path does not point to a folder: '{}'",
            solution_dir.display().underline()
        );

        let problem_id = resolve_and_get_file_name(&solution_dir)
            .wrap_err("Failed to extract problem ID from the solution folder")?;

        let solution_file = resolve_solution_file_to_use(app, &solution_dir, &options)?;

        let solution_lang = options
            .lang
            .map_or_else(
                || app.config.lang_from_file(&solution_file),
                |ext| Ok(app.config.lang_from_file_ext(ext)),
            )?
            .ok_or_else(|| eyre::eyre!("kitty doesn't recognise the language"))?;

        Ok(Self {
            id: problem_id,
            dir: solution_dir,
            file: solution_file,
            lang: solution_lang,
        })
    }

    pub fn debug_dir(&self) -> PathBuf {
        self.dir.join("debug")
    }

    pub fn debug_save_dir(&self) -> PathBuf {
        self.debug_dir().join("saved")
    }
}

#[derive(Debug)]
pub struct SolutionOptions<'a> {
    pub file_path: Option<&'a PathBuf>,
    pub lang: Option<&'a String>,
}

pub fn get_test_dir(solution_dir: impl AsRef<Path>) -> PathBuf {
    solution_dir.as_ref().join("test")
}

fn resolve_solution_file_to_use(
    app: &App,
    solution_dir: impl AsRef<Path>,
    options: &SolutionOptions,
) -> crate::Result<PathBuf> {
    if let Some(file_path) = &options.file_path {
        let file_path = file_path.as_path();

        eyre::ensure!(
            file_path.is_file(),
            "The solution file path does not point to a file: '{}'",
            file_path.display().underline()
        );

        return Ok(file_path.to_path_buf());
    }

    let solution_dir = solution_dir.as_ref();
    let options = get_all_files_with_known_extension(app, solution_dir)?;

    eyre::ensure!(
        !options.is_empty(),
        "No solution files found in the solution folder: {}",
        solution_dir.display().underline()
    );

    if let [file] = options.as_slice() {
        return Ok(file.clone());
    }

    let file_names = options
        .iter()
        .map(resolve_and_get_file_name)
        .collect::<crate::Result<Vec<_>>>()
        .wrap_err("Failed to extract file names from solution folder")?;

    let selection = dialoguer::Select::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .with_prompt("Multiple solution files found. Specify which file to use:")
        .items(&file_names)
        .default(0)
        .interact()
        .wrap_err("Failed to choose solution file")?;

    options
        .get(selection)
        .cloned()
        .wrap_err("Selected option was invalid")
}

pub fn get_all_files_with_known_extension(
    app: &App,
    folder: impl AsRef<Path>,
) -> crate::Result<Vec<PathBuf>> {
    let all_entries = fs::read_dir(folder).wrap_err("Failed to read solution folder contents")?;

    let options = all_entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_file())
        .filter(|path| {
            path.extension()
                .and_then(|ext| ext.to_str())
                .and_then(|ext| app.config.lang_from_file_ext(ext))
                .is_some()
        })
        .collect();

    Ok(options)
}

pub fn get_test_cases(solution_dir: impl AsRef<Path>) -> crate::Result<Vec<FileTestCase>> {
    let test_dir_files = get_test_dir(solution_dir)
        .read_dir()
        .wrap_err("Failed to read test case folder")?
        .filter_map(Result::ok)
        .filter(|entry| {
            entry
                .file_type()
                .ok()
                .map(|file_type| file_type.is_file())
                .unwrap_or(false)
        })
        .map(|entry| entry.path());

    // We use a BTreeMap to easily sort the test cases by name
    let mut input_files = BTreeMap::new();
    let mut answer_files = BTreeMap::new();

    for file in test_dir_files {
        let extension = file
            .extension()
            .and_then(OsStr::to_str)
            .map(str::to_lowercase);

        let map = match extension.as_deref() {
            Some("in") => &mut input_files,
            Some("ans") => &mut answer_files,
            _ => continue,
        };

        if let Some(name) = file.file_stem().and_then(OsStr::to_str) {
            map.insert(name.to_owned(), file);
        }
    }

    let test_files = input_files
        .into_iter()
        .filter_map(|(name, input_file)| {
            answer_files.remove(&name).map(|answer_file| FileTestCase {
                name,
                input_file,
                answer_file,
            })
        })
        .collect();

    Ok(test_files)
}
