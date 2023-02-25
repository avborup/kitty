use std::path::Path;

use eyre::Context;
use ini::Ini;
use secrecy::Secret;

#[derive(Debug)]
pub struct Kattisrc {
    pub user: Credentials,
    pub kattis: KattisSettings,
}

#[derive(Debug)]
pub struct Credentials {
    pub username: String,
    pub token: Secret<String>,
}

#[derive(Debug)]
pub struct KattisSettings {
    pub host_name: String,
    pub login_url: String,
    pub submission_url: String,
    pub submissions_url: String,
}

impl Kattisrc {
    pub fn from_file(path: impl AsRef<Path>) -> crate::Result<Option<Self>> {
        let path = path.as_ref();

        if !path.exists() {
            return Ok(None);
        }

        let ini = Ini::load_from_file(path)
            .with_context(|| format!("failed to read .kattisrc at '{}'", path.display()))?;

        let kattisrc = Self::from_ini(&ini)?;

        Ok(Some(kattisrc))
    }

    pub fn from_ini(ini: &Ini) -> crate::Result<Self> {
        fn get_field(ini: &Ini, section: &str, field: &str) -> crate::Result<String> {
            let value = ini
                .section(Some(section))
                .and_then(|section| section.get(field))
                .ok_or_else(|| {
                    eyre::eyre!(".kattisrc is missing field '{field}' in section [{section}]")
                })?;

            Ok(value.to_string())
        }

        let user = Credentials {
            username: get_field(ini, "user", "username")?,
            token: Secret::new(get_field(ini, "user", "token")?),
        };

        let kattis = KattisSettings {
            host_name: get_field(ini, "kattis", "hostname")?,
            login_url: get_field(ini, "kattis", "loginurl")?,
            submission_url: get_field(ini, "kattis", "submissionurl")?,
            submissions_url: get_field(ini, "kattis", "submissionsurl")?,
        };

        Ok(Self { user, kattis })
    }
}
