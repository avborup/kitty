use std::{
    env::{self, consts::EXE_EXTENSION},
    path::{Path, PathBuf},
};

use eyre::ensure;
use kattisrc::Kattisrc;

use crate::utils::get_full_path;

use self::{language::Language, parser::parse_config_from_yaml_file};

pub mod kattisrc;
pub mod language;
mod parser;

#[derive(Debug, Default)]
pub struct Config {
    pub kattisrc: Option<Kattisrc>,
    pub config_dir: PathBuf,
    pub default_language: Option<String>,
    pub languages: Vec<Language>,
}

impl Config {
    pub fn load() -> crate::Result<Self> {
        let config_dir =
            Self::get_config_dir_path_from_env()?.unwrap_or_else(Self::default_config_dir_path);

        let kattisrc = Kattisrc::from_file(Self::kattisrc_path_with_dir(&config_dir))?;
        let yml_config = parse_config_from_yaml_file(Self::config_file_path_with_dir(&config_dir))?;

        let config = Config {
            kattisrc,
            config_dir,
            ..yml_config
        };

        Ok(config)
    }

    pub fn try_kattisrc(&self) -> crate::Result<&Kattisrc> {
        self.kattisrc
            .as_ref()
            .ok_or_else(|| eyre::eyre!("Could not find .kattisrc file. You must download your .kattisrc file from https://open.kattis.com/download/kattisrc and save it at '{}'", self.kattisrc_path().display()))
    }

    pub fn get_config_dir_path_from_env() -> crate::Result<Option<PathBuf>> {
        match env::var("KATTIS_KITTY_CONFIG_DIR").map(PathBuf::from) {
            Ok(path) => {
                ensure!(
                    path.is_dir(),
                    "The config directory path '{}' is not a directory",
                    path.display()
                );

                Ok(Some(path))
            }
            Err(_) => Ok(None),
        }
    }

    /// Gets kitty's config directory. The location of this directory will vary
    /// by platform:
    ///  - `%APPDATA%/kitty` on Windows
    ///  - `~/.config/kitty` on Linux
    ///  - `~/Library/Application Support/kitty` on macOS
    pub fn default_config_dir_path() -> PathBuf {
        platform_dirs::AppDirs::new(Some("kitty"), false)
            .expect("failed to find where the kitty config directory should be located")
            .config_dir
    }

    pub fn kattisrc_path(&self) -> PathBuf {
        Self::kattisrc_path_with_dir(&self.config_dir)
    }

    pub fn config_file_path(&self) -> PathBuf {
        Self::config_file_path_with_dir(&self.config_dir)
    }

    pub fn templates_dir_path(&self) -> PathBuf {
        Self::templates_dir_path_with_dir(&self.config_dir)
    }

    pub fn kattisrc_path_with_dir(dir: impl AsRef<Path>) -> PathBuf {
        dir.as_ref().join(".kattisrc")
    }

    pub fn config_file_path_with_dir(dir: impl AsRef<Path>) -> PathBuf {
        dir.as_ref().join("kitty.yml")
    }

    pub fn templates_dir_path_with_dir(dir: impl AsRef<Path>) -> PathBuf {
        dir.as_ref().join("templates")
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
                .replace("$DIR_PATH", &path_to_str(dir_path)?)
                .replace("$EXE_PATH", &path_to_str(&exe_path)?);

            Ok(populated)
        })
        .collect::<crate::Result<_>>();

    parts
}
