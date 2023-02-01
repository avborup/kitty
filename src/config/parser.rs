use std::{fs, path::Path};

use crate::{config::language::Language, config::Config};
use eyre::Context;
use yaml_rust::{Yaml, YamlLoader};

#[cfg(unix)]
const PLATFORM_KEY: &str = "unix";
#[cfg(windows)]
const PLATFORM_KEY: &str = "windows";

pub fn parse_config_from_yaml_file(path: impl AsRef<Path>) -> crate::Result<Config> {
    let path = path.as_ref();

    if !path.exists() {
        return Ok(Config::default());
    }

    let config_str = fs::read_to_string(path)
        .wrap_err_with(|| format!("Failed to read config file at '{}'", path.display()))?;

    parse_config_from_yaml(&config_str)
}

pub fn parse_config_from_yaml(yaml_str: &str) -> crate::Result<Config> {
    let docs = YamlLoader::load_from_str(yaml_str).wrap_err("Failed to parse kitty config file")?;

    let doc = match docs.first() {
        Some(d) => d,
        None => return Ok(Default::default()),
    };

    let default_language = doc["default_language"].as_str().map(str::to_lowercase);
    let languages = doc["languages"]
        .as_vec()
        .map(|v| v.iter().map(lang_from_yml).collect::<Result<Vec<_>, _>>())
        .unwrap_or_else(|| Ok(Vec::new()))?;

    let config = Config {
        default_language,
        languages,
        ..Default::default()
    };

    Ok(config)
}

fn lang_from_yml(lang_block: &Yaml) -> crate::Result<Language> {
    fn map_err_with_name(name: &str, err: eyre::Report) -> eyre::Report {
        eyre::eyre!("Failed to read language configuration for '{name}': {err}")
    }

    let name = get_value_else_err("name", lang_block)?;
    let file_ext = get_value_else_err("file_extension", lang_block)
        .map_err(|e| map_err_with_name(&name, e))?
        .to_lowercase();
    let run_cmd =
        get_value_else_err("run_command", lang_block).map_err(|e| map_err_with_name(&name, e))?;
    let compile_cmd = get_string_value("compile_command", lang_block);

    Ok(Language::new(name, file_ext, run_cmd, compile_cmd))
}

fn get_value_else_err(key: &str, doc: &Yaml) -> crate::Result<String> {
    get_string_value(key, doc)
        .ok_or_else(|| eyre::eyre!("Languages in the config file must contain a '{key}' field"))
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
