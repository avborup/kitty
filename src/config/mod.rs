use std::path::PathBuf;

use kattisrc::Kattisrc;

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
        self.languages.iter().find(|l| l.file_ext() == file_ext)
    }

    pub fn default_language(&self) -> Option<&Language> {
        self.default_language
            .as_ref()
            .and_then(|l| self.lang_from_file_ext(l))
    }
}
