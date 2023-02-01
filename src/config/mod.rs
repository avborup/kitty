use std::path::PathBuf;

use kattisrc::Kattisrc;

mod kattisrc;

#[derive(Debug)]
pub struct Config {
    pub kattisrc: Option<Kattisrc>,
}

impl Config {
    pub fn load() -> crate::Result<Self> {
        let kattisrc = Kattisrc::from_file(Self::kattisrc_path())?;

        let config = Config { kattisrc };

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

    fn kattisrc_path() -> PathBuf {
        Self::dir_path().join(".kattisrc")
    }
}
