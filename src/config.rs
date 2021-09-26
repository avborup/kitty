use crate::lang::Language;
use crate::utils::path_to_str;
use crate::StdErr;
use ini::{Ini, Properties};
use platform_dirs::AppDirs;
use std::env::consts::EXE_EXTENSION;
use std::fmt::Debug;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Default, Debug)]
pub struct Config {
    default_language: Option<String>,
    languages: Vec<Language>,
    kattisrc: Option<Kattisrc>,
}

impl Config {
    pub fn load() -> Result<Self, StdErr> {
        let config_file = Self::dir_path().join("kitty.yml");
        let kattisrc = Kattisrc::load()?;

        if !config_file.exists() {
            return Ok(Self {
                kattisrc,
                ..Default::default()
            });
        }

        let config_text = fs::read_to_string(config_file)?;
        let config = config_parser::parse_config_from_yaml(&config_text)?;

        Ok(Self { kattisrc, ..config })
    }

    pub fn lang_from_file_ext(&self, file_ext: &str) -> Option<&Language> {
        self.languages
            .iter()
            .find(|l| l.file_ext().to_lowercase() == file_ext.to_lowercase())
    }

    pub fn lang_from_file(&self, file: &Path) -> Result<Option<&Language>, StdErr> {
        let ext = match file.extension() {
            Some(e) => e.to_str().expect("invalid unicode in file extension"),
            None => return Err("file has no file extension".into()),
        };

        let lang = self.lang_from_file_ext(ext);

        Ok(lang)
    }

    pub fn languages(&self) -> impl Iterator<Item = &Language> {
        self.languages.iter()
    }

    pub fn default_language(&self) -> Option<&Language> {
        self.default_language
            .as_ref()
            .and_then(|l| self.lang_from_file_ext(l))
    }

    /// Gets kitty's config directory. The location of this directory will vary
    /// by platform:
    ///  - `%APPDATA%/kitty` on Windows
    ///  - `~/.config/kitty` on Linux
    ///  - `~/Library/Application Support/kitty` on macOS
    pub fn dir_path() -> PathBuf {
        AppDirs::new(Some("kitty"), false)
            .expect("failed to find where the kitty config directory should be located")
            .config_dir
    }

    /// Retrieves the path to the directory containing user-defined templates.
    pub fn templates_dir_path() -> PathBuf {
        Self::dir_path().join("templates")
    }

    /// Sets up the config directory and gives back the path of it.
    pub fn init() -> io::Result<PathBuf> {
        let dir_path = Self::dir_path();
        fs::create_dir_all(Self::templates_dir_path())?;
        Ok(dir_path)
    }

    pub fn kattisrc(&self) -> Result<&Kattisrc, StdErr> {
        self.kattisrc.as_ref().ok_or_else(|| {
            format!("could not find .kattisrc file. you must download your .kattisrc file from https://open.kattis.com/download/kattisrc and save it at {}",
                Kattisrc::path().to_str().expect("config file path contained invalid unicode")).into()
        })
    }
}

mod config_parser {
    use crate::{config::Config, lang::Language, StdErr};
    use yaml_rust::{Yaml, YamlLoader};

    #[cfg(unix)]
    const PLATFORM_KEY: &str = "unix";
    #[cfg(windows)]
    const PLATFORM_KEY: &str = "windows";

    pub fn parse_config_from_yaml(yaml_str: &str) -> Result<Config, StdErr> {
        let docs = YamlLoader::load_from_str(yaml_str)?;

        let doc = match docs.first() {
            Some(d) => d,
            None => return Ok(Default::default()),
        };

        let default_language = doc["default_language"].as_str().map(str::to_string);
        let languages = doc["languages"]
            .as_vec()
            .map(|v| {
                v.iter()
                    .map(|b| lang_from_yml(b))
                    .collect::<Result<Vec<_>, _>>()
            })
            .unwrap_or_else(|| Ok(Vec::new()))?;

        let config = Config {
            default_language,
            languages,
            ..Default::default()
        };

        Ok(config)
    }

    fn lang_from_yml(lang_block: &Yaml) -> Result<Language, StdErr> {
        let name = get_value_else_err("name", lang_block)?;
        let file_ext = get_value_else_err("file_extension", lang_block)?;
        let run_cmd = get_value_else_err("run_command", lang_block)?;
        let compile_cmd = get_string_value("compile_command", lang_block);

        Ok(Language::new(name, file_ext, run_cmd, compile_cmd))
    }

    fn get_value_else_err(key: &str, doc: &Yaml) -> Result<String, StdErr> {
        get_string_value(key, doc).ok_or_else(|| {
            format!(
                "languages in the config file must contain a '{}' field",
                key,
            )
            .into()
        })
    }

    fn get_string_value(key: &str, doc: &Yaml) -> Option<String> {
        get_value(key, doc).and_then(|y| y.into_string())
    }

    fn get_value(key: &str, doc: &Yaml) -> Option<Yaml> {
        let platform_value = &doc[PLATFORM_KEY][key];

        Some(if !platform_value.is_badvalue() {
            platform_value.clone()
        } else {
            doc[key].clone()
        })
    }
}

pub fn prepare_cmd(cmd: &str, file_path: &Path) -> Option<Vec<String>> {
    let mut dir_path = file_path.to_path_buf();
    dir_path.pop();
    let exe_path = file_path.with_extension(EXE_EXTENSION);
    let file_name_no_ext = file_path.file_stem().unwrap().to_str().unwrap();

    shlex::split(cmd).map(|args| {
        args.iter()
            .map(|arg| {
                arg.replace("$SRC_PATH", &path_to_str(file_path))
                    .replace("$SRC_FILE_NAME_NO_EXT", file_name_no_ext)
                    .replace("$DIR_PATH", &path_to_str(&dir_path))
                    .replace("$EXE_PATH", &path_to_str(&exe_path))
            })
            .collect()
    })
}

pub struct Kattisrc {
    ini: Ini,
}

impl Kattisrc {
    pub fn load() -> Result<Option<Self>, StdErr> {
        let kattisrc_path = Self::path();

        if !kattisrc_path.exists() {
            return Ok(None);
        }

        let ini = Ini::load_from_file(&kattisrc_path)?;

        Ok(Some(Self { ini }))
    }

    pub fn path() -> PathBuf {
        let config_dir = Config::dir_path();
        config_dir.join(".kattisrc")
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
}

impl Debug for Kattisrc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut ini_buf = Vec::new();
        self.ini
            .write_to(&mut ini_buf)
            .expect("failed to write ini bytes");
        let ini_str = String::from_utf8(ini_buf).expect("failed to write ini to utf-8 string");

        f.debug_struct("Kattisrc").field("ini", &ini_str).finish()
    }
}

/// A wrapper for a username and a secret token.
#[derive(Clone)]
pub struct Credentials {
    pub username: String,
    pub token: String,
}
