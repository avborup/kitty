use std::{
    env::consts::EXE_EXTENSION,
    path::{Path, PathBuf},
};

use kattisrc::Kattisrc;

use crate::utils::get_full_path;

use self::{language::Language, parser::parse_config_from_yaml_file};

mod kattisrc;
pub mod language;
mod parser;

#[derive(Debug, Default)]
pub struct Config {
    pub kattisrc: Option<Kattisrc>,
    pub default_language: Option<String>,
    pub languages: Vec<Language>,
}

impl Config {
    pub fn load() -> crate::Result<Self> {
        let kattisrc = Kattisrc::from_file(Self::kattisrc_path())?;
        let yml_config = parse_config_from_yaml_file(Self::config_file_path())?;

        let config = Config {
            kattisrc,
            ..yml_config
        };

        Ok(config)
    }

    pub fn try_kattisrc(&self) -> crate::Result<&Kattisrc> {
        self.kattisrc
            .as_ref()
            .ok_or_else(|| eyre::eyre!("Could not find .kattisrc file. You must download your .kattisrc file from https://open.kattis.com/download/kattisrc and save it at '{}'", Self::kattisrc_path().display()))
    }

    /// Gets kitty's config directory. The location of this directory will vary
    /// by platform:
    ///  - `%APPDATA%/kitty` on Windows
    ///  - `~/.config/kitty` on Linux
    ///  - `~/Library/Application Support/kitty` on macOS
    pub fn dir_path() -> PathBuf {
        platform_dirs::AppDirs::new(Some("kitty"), false)
            .expect("failed to find where the kitty config directory should be located")
            .config_dir
    }

    pub fn kattisrc_path() -> PathBuf {
        Self::dir_path().join(".kattisrc")
    }

    pub fn config_file_path() -> PathBuf {
        Self::dir_path().join("kitty.yml")
    }

    pub fn templates_dir_path() -> PathBuf {
        Self::dir_path().join("templates")
    }

    pub fn lang_from_file_ext(&self, file_ext: &str) -> Option<&Language> {
        self.languages
            .iter()
            .find(|l| l.file_ext().to_lowercase() == file_ext.to_lowercase())
    }

    pub fn lang_from_file(&self, file: impl AsRef<Path>) -> crate::Result<Option<&Language>> {
        let ext = file
            .as_ref()
            .extension()
            .and_then(|ext| ext.to_str())
            .ok_or_else(|| eyre::eyre!("File has no extension"))?;

        Ok(self.lang_from_file_ext(ext))
    }

    pub fn default_language(&self) -> Option<&Language> {
        self.default_language
            .as_ref()
            .and_then(|l| self.lang_from_file_ext(l))
    }
}

pub fn prepare_cmd(cmd: &str, file_path: impl AsRef<Path>) -> crate::Result<Vec<String>> {
    fn path_to_str(path: impl AsRef<Path>) -> crate::Result<String> {
        path.as_ref()
            .to_str()
            .map(str::to_string)
            .ok_or_else(|| eyre::eyre!("Could not convert path to string"))
    }

    let file_path = get_full_path(file_path)?;

    let dir_path = file_path
        .parent()
        .ok_or_else(|| eyre::eyre!("Could not find parent of path '{}'", file_path.display()))?;

    let exe_path = file_path.with_extension(EXE_EXTENSION);
    let file_name_no_ext = file_path.file_stem().unwrap().to_str().unwrap();

    let parts = shlex::split(cmd)
        .ok_or_else(|| eyre::eyre!("Could not parse command"))?
        .iter()
        .map(|arg| {
            let populated = arg
                .replace("$SRC_PATH", &path_to_str(&file_path)?)
                .replace("$SRC_FILE_NAME_NO_EXT", file_name_no_ext)
                .replace("$DIR_PATH", &path_to_str(&dir_path)?)
                .replace("$EXE_PATH", &path_to_str(&exe_path)?);

            Ok(populated)
        })
        .collect::<crate::Result<_>>();

    parts
}
