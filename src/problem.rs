use regex::Regex;

use crate::App;

pub fn make_problem_url(app: &App, problem_id: &str) -> crate::Result<String> {
    let host_name = &app.config.try_kattisrc()?.kattis.host_name;
    let url = format!("https://{host_name}/problems/{problem_id}");
    Ok(url)
}

pub fn make_problem_sample_tests_zip_url(app: &App, problem_id: &str) -> crate::Result<String> {
    let problem_url = make_problem_url(app, problem_id)?;
    let zip_url = format!("{problem_url}/file/statement/samples.zip");
    Ok(zip_url)
}

pub fn problem_id_is_legal(problem_id: &str) -> bool {
    Regex::new(r"^[\w\d\.]+$").unwrap().is_match(problem_id)
}
