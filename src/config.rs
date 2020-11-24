use crate::lang::Language;
use crate::StdErr;
use ini::{Ini, Properties, SectionSetter};
use platform_dirs::AppDirs;
use std::fs;
use std::io;
use std::path::PathBuf;

/// A configuration interaction layer.
///
/// The associated methods wrap the Ini struct from [rust-ini], and construct
/// errors and return values that are appropriate for kitty.
///
/// [rust-ini]: https://crates.io/crates/rust-ini
pub struct Config {
    ini: Ini,
    dir: PathBuf,
    file: PathBuf,
}

impl Config {
    /// Instantiates a `Config` by loading the `.kattisrc` file located at
    /// kitty's config directory. The location of this directory will vary by
    /// platform:
    ///  - `%APPDATA%/kitty` on Windows
    ///  - `~/.config/kitty` on Linux
    ///  - `~/Library/Application Support/kitty` on macOS
    ///
    /// Fails if the file does not exist or if the file cannot be read.
    pub fn load() -> Result<Self, StdErr> {
        let config_dir = Self::dir_path();
        let config_file = config_dir.join(".kattisrc");

        if !config_file.exists() {
            return Err(format!("could not find .kattisrc file. you must download your .kattisrc file from https://open.kattis.com/download/kattisrc and save it at {}",
                               config_file.to_str().expect("config file path contained invalid unicode")).into());
        }

        let cfg = match Ini::load_from_file(&config_file) {
            Ok(c) => c,
            Err(_) => return Err("failed to read .kattisrc file".into()),
        };

        Ok(Self {
            ini: cfg,
            dir: config_dir,
            file: config_file,
        })
    }

    /// Sets up the config directory and gives back the path of it.
    pub fn init() -> io::Result<PathBuf> {
        let dir_path = Self::dir_path();
        fs::create_dir_all(dir_path.join("templates"))?;

        Ok(dir_path)
    }

    /// Gets the config directory path depending on platform.
    pub fn dir_path() -> PathBuf {
        AppDirs::new(Some("kitty"), false)
            .expect("failed to find where the kitty config directory should be located")
            .config_dir
    }

    /// Retrieves credentials from the config if the config file contains the
    /// following section:
    /// ```ini
    /// [user]
    /// username: <some username>
    /// token: <some token>
    /// ```
    /// If this is not the case, `Err` is returned.
    pub fn get_credentials(&self) -> Result<Credentials, StdErr> {
        let user_section = match self.ini.section(Some("user")) {
            Some(u) => u,
            None => return Err("could not find user section in .kattisrc".into()),
        };

        let username = match user_section.get("username") {
            Some(u) => u,
            None => return Err("could not find username under [user] in .kattisrc".into()),
        };

        let token = match user_section.get("token") {
            Some(u) => u,
            None => return Err("could not find token under [user] in .kattisrc".into()),
        };

        Ok(Credentials {
            username: username.to_string(),
            token: token.to_string(),
        })
    }

    fn get_kattis_section(&self) -> Result<&Properties, StdErr> {
        let kattis_section = match self.ini.section(Some("kattis")) {
            Some(k) => k,
            None => return Err("could not find kattis section in .kattisrc".into()),
        };

        Ok(kattis_section)
    }

    /// Retrieves the host name from the config if the config file contains the
    /// following section:
    /// ```ini
    /// [kattis]
    /// hostname: <e.g. open.kattis.com>
    /// ```
    /// If this cannot be found, `Err` is returned.
    pub fn get_host_name(&self) -> Result<&str, StdErr> {
        match self.get_kattis_section()?.get("hostname") {
            Some(u) => Ok(u),
            None => Err("could not find hostname under [kattis] in .kattisrc".into()),
        }
    }

    /// Retrieves the submission url from the config if the config file contains
    /// the following section:
    /// ```ini
    /// [kattis]
    /// submissionurl: <e.g. https://open.kattis.com/submit>
    /// ```
    /// If this cannot be found, `Err` is returned.
    pub fn get_submit_url(&self) -> Result<&str, StdErr> {
        match self.get_kattis_section()?.get("submissionurl") {
            Some(u) => Ok(u),
            None => Err("could not find submission url under [kattis] in .kattisrc".into()),
        }
    }

    /// Retrieves the submissions url from the config if the config file
    /// contains the following section:
    /// ```ini
    /// [kattis]
    /// submissionsurl: <e.g. https://open.kattis.com/submissions>
    /// ```
    /// If this cannot be found, `Err` is returned.
    pub fn get_submissions_url(&self) -> Result<&str, StdErr> {
        match self.get_kattis_section()?.get("submissionsurl") {
            Some(u) => Ok(u),
            None => Err("could not find submissions url under [kattis] in .kattisrc".into()),
        }
    }

    /// Retrieves the login url from the config if the config file contains the
    /// following section:
    /// ```ini
    /// [kattis]
    /// loginurl: <e.g. https://open.kattis.com/login>
    /// ```
    /// If this cannot be found, `Err` is returned.
    pub fn get_login_url(&self) -> Result<&str, StdErr> {
        match self.get_kattis_section()?.get("loginurl") {
            Some(u) => Ok(u),
            None => Err("could not find login url under [kattis] in .kattisrc".into()),
        }
    }

    /// Retrieves the path to the directory containing user-defined templates.
    pub fn get_templates_dir(&self) -> PathBuf {
        self.dir.join("templates")
    }

    /// Retrieves the section from the config file with "kitty" as the header if
    /// it exists.
    fn get_kitty_section(&self) -> Option<&Properties> {
        self.ini.section(Some("kitty"))
    }

    /// Retrieves a mutable section from the config file if it exists or creates
    /// a mutable section into which key/value pairs can be added.
    fn get_kitty_section_mut(&mut self) -> SectionSetter {
        self.ini.with_section(Some("kitty"))
    }

    /// Writes the config to the config file.
    pub fn save(&self) -> Result<(), StdErr> {
        self.ini.write_to_file(&self.file)?;

        Ok(())
    }

    /// Adds or overwrites the default language setting.
    pub fn set_default_lang(&mut self, lang: &Language) {
        let mut kitty_section = self.get_kitty_section_mut();
        kitty_section.set("default_language", lang.file_ext());
    }

    /// Retrieves the default language setting from the config file.
    pub fn get_default_lang(&self) -> Option<Language> {
        self.get_kitty_section()
            .and_then(|s| s.get("default_language"))
            .map(|l| Language::from_file_ext(l))
    }
}

/// A wrapper for a username and a secret token.
#[derive(Clone)]
pub struct Credentials {
    pub username: String,
    pub token: String,
}
