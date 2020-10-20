use clap::ArgMatches;
use std::env;
use std::fs;
use std::io::{self, Read, Write};
use std::path::PathBuf;
use zip::ZipArchive;
use colored::Colorize;
use crate::config::Config;
use crate::StdErr;

pub async fn get(cmd: &ArgMatches<'_>) -> Result<(), StdErr> {
    // We can unwrap here because clap will exit automatically when this arg is
    // not present.
    let id = cmd.value_of("PROBLEM ID").unwrap();

    if !id.chars().all(char::is_alphanumeric) {
        return Err("problem id must only contain alphanumeric characters".into());
    }

    let cfg = Config::load()?;
    let host_name = cfg.get_host_name()?;

    let p_url = format!("https://{}/problems/{}", host_name, id);
    let p_res = reqwest::get(&p_url).await?;

    let p_status = p_res.status();
    if !p_status.is_success() {
        match p_status.as_str() {
            "404" => return Err(format!("the problem \"{}\" does not exist", id).into()),
            _ => return Err(format!("failed to fetch problem \"{}\" (http status code {})", id, p_status).into()),
        }
    }

    let cwd = env::current_dir()?;
    let p_dir = cwd.join(id);

    if let Err(e) = fs::create_dir(&p_dir) {
        return match e.kind() {
            io::ErrorKind::AlreadyExists => Err("cannot create problem directory since it already exists".into()),
            _ => Err("failed to create problem directory at this location".into()),
        }
    }

    fetch_tests(&p_dir, &p_url).await?;

    println!("{} problem \"{}\"", "created".bright_green(), id);

    Ok(())
}

async fn fetch_tests(parent_dir: &PathBuf, problem_url: &str) -> Result<(), StdErr> {
    let t_dir = parent_dir.join("test");
    let t_dir = t_dir.as_path();
    if let Err(_) = fs::create_dir(t_dir) {
        return Err("failed to create problem directory at this location".into());
    }

    let zip_url = format!("{}/file/statement/samples.zip", problem_url);
    let z_res = reqwest::get(&zip_url).await?;

    let z_status = z_res.status();
    if !z_status.is_success() {
        return match z_status.as_str() {
            "404" => Ok(()),
            _ => Err(format!("failed to fetch tests (http status code {})", z_status).into()),
        }
    }

    let mut tmpfile = match tempfile::tempfile() {
        Ok(t) => t,
        Err(_) => return Err("failed to create temporary file for storing test samples".into()),
    };

    if let Err(_) = tmpfile.write_all(&z_res.bytes().await?) {
        return Err("failed to write test samples to temporary zip file".into());
    }

    let mut zip = match ZipArchive::new(tmpfile) {
        Ok(z) => z,
        Err(_) => return Err("failed to open zip file with test samples".into()),
    };

    for i in 0..zip.len() {
        let mut file = match zip.by_index(i) {
            Ok(f) => f,
            Err(_) => return Err("failed to read zip file with test samples".into()),
        };

        if file.is_dir() {
            return Err("unexpected directory in test samples".into());
        }

        let name = file.name().to_owned();
        let file_path = t_dir.join(&name);
        let mut dest = match fs::File::create(&file_path) {
            Ok(f) => f,
            Err(_) => return Err(format!("failed to create sample file {}", &name).into()),
        };

        let mut content = String::new();
        if let Err(_) = file.read_to_string(&mut content) {
            return Err("failed to read sample file from zip".into());
        }

        if let Err(_) = dest.write_all(&content.as_bytes()) {
            return Err(format!("failed to write to file {}", &name).into());
        }
    }

    Ok(())
}
