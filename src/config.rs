use platform_dirs::AppDirs;
use ini::Ini;
use crate::StdErr;

/// A configuration interaction layer.
///
/// The associated methods wrap the Ini struct from [rust-ini], and construct
/// errors and return values that are appropriate for kitty.
///
/// [rust-ini]: https://crates.io/crates/rust-ini
pub struct Config {
    ini: Ini,
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
        let app_dirs = match AppDirs::new(Some("kitty"), false) {
            Some(a) => a,
            None => return Err("failed to find kitty config directory".into()),
        };
        let config_path = app_dirs.config_dir.join(".kattisrc");

        if !config_path.exists() {
            return Err(format!("could not find .kattisrc file. you must download your .kattisrc file from https://open.kattis.com/download/kattisrc and save it at {}",
                               config_path.to_str().expect("config file path contained invalid unicode")).into());
        }

        let cfg = match Ini::load_from_file(&config_path) {
            Ok(c) => c,
            Err(_) => return Err("failed to read .kattisrc file".into()),
        };

        Ok(Self {
            ini: cfg,
        })
    }

    /// Retrieves credentials from the config if the config file contains the
    /// following section:
    /// ```ini
    /// [user]
    /// username = <some username>
    /// token = <some token>
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
}

/// A wrapper for a username and a secret token.
#[derive(Clone)]
pub struct Credentials {
    pub username: String,
    pub token: String,
}
