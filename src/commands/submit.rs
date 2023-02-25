use std::{
    fs,
    io::{self, Write},
    time::Duration,
};

use colored::Colorize;
use eyre::Context;
use regex::Regex;
use reqwest::multipart::{Form, Part};
use scraper::{node::Element, Html, Selector};
use selectors::attr::CaseSensitivity;
use serde::Deserialize;
use tokio::time::sleep;

use crate::{
    cli::SubmitArgs,
    solution::{Solution, SolutionOptions},
    utils::{prompt_bool, resolve_and_get_file_name},
    App,
};

const REQUEST_INTERVAL_DURATION: Duration = Duration::from_millis(250);

const SUCCESS: &str = "🟢";
const FAILURE: &str = "🔴";
const UNKNOWN: &str = "⚪";

pub async fn submit(app: &App, args: &SubmitArgs) -> crate::Result<()> {
    let solution = Solution::from_folder(
        app,
        &args.path,
        SolutionOptions {
            file_path: args.file.as_ref(),
            lang: args.lang.as_ref(),
        },
    )?;

    let file_name = resolve_and_get_file_name(&solution.file)?;

    println!("{}:  {}", "Problem".bright_cyan(), &solution.id);
    println!("{}: {}", "Language".bright_cyan(), &solution.lang);
    println!("{}:     {}", "File".bright_cyan(), &file_name);

    if !args.yes && !prompt_bool("Should this be submitted?")? {
        return Ok(());
    }

    app.client.login(app).await?;

    let submission_id = submit_solution(app, &solution).await?;

    let submission_url = make_submission_url(app, &submission_id)?;
    println!(
        "{} solution to {}",
        "Submitted".bright_green(),
        &submission_url.underline()
    );

    if args.open {
        webbrowser::open(&submission_url).wrap_err("Failed to open submission in browser")?;
    }

    show_submission_status(app, &submission_id).await?;

    Ok(())
}

fn make_submission_url(app: &App, submission_id: &str) -> crate::Result<String> {
    Ok(format!(
        "{}/{submission_id}",
        app.config.try_kattisrc()?.kattis.submissions_url,
    ))
}

async fn submit_solution(app: &App, solution: &Solution<'_>) -> crate::Result<String> {
    let kattisrc = app.config.try_kattisrc()?;
    let file_name = resolve_and_get_file_name(&solution.file)?;

    // This will work for the vast majority of cases, but it may fail in some
    // instances. It has to be improved in the future if there is any problem.
    // Kattis doesn't care about mainclass if the programming language doesn't
    // need it (for example Python), so whatever we set doesn't matter. For the
    // languages where it is required (for example Java), the file name is very
    // likely to be the same as the main class name.
    let main_class = file_name.split('.').next().unwrap_or_default().to_string();

    let file_bytes = fs::read(&solution.file).wrap_err("Failed to read solution file")?;
    let file_part = Part::bytes(file_bytes)
        .file_name(file_name)
        .mime_str("application/octet-stream")?;

    let form = Form::new()
        .text("problem", solution.id.clone())
        .text("language", solution.lang.to_string())
        .part("sub_file[]", file_part)
        .text("mainclass", main_class)
        .text("submit_ctr", "2")
        .text("submit", "true")
        .text("script", "true");

    let res = app
        .client
        .post(&kattisrc.kattis.submission_url)
        .multipart(form)
        .send()
        .await
        .wrap_err("Failed to send request to Kattis")?;

    if !res.status().is_success() {
        eyre::bail!(
            "Failed to submit solution to Kattis (http status code: {})",
            res.status()
        )
    }

    let res_body = res
        .text()
        .await
        .wrap_err("Failed to read response from Kattis")?;

    if res_body.contains("Problem not found") {
        eyre::bail!("The problem '{}' does not exist", solution.id)
    }

    let submission_id_regex = Regex::new(r"ID: (\d+)").unwrap();
    let submission_id = submission_id_regex
        .captures(&res_body)
        .and_then(|c| c.get(1))
        .map(|i| i.as_str().to_string())
        .ok_or_else(|| {
            eyre::eyre!(
                "Failed to find submission ID in response from Kattis. Received: {res_body}"
            )
        })?;

    Ok(submission_id)
}

async fn show_submission_status(app: &App, submission_id: &str) -> crate::Result<()> {
    println!();

    loop {
        let status = get_submission_status(app, submission_id)
            .await
            .wrap_err("Failed to get submission status from Kattis")?;

        if let SubmissionStage::BeforeTests(stage) = &status.verdict {
            print!("\r{}: {stage}", "Status".bright_cyan());
        } else {
            let test_cases_str = status
                .test_cases
                .iter()
                .map(|tc| match tc {
                    TestCaseStatus::Accepted => SUCCESS,
                    TestCaseStatus::Rejected(_) => FAILURE,
                    TestCaseStatus::Unfinished => UNKNOWN,
                })
                .collect::<Vec<_>>()
                .join("");

            let num_completed = status
                .test_cases
                .iter()
                .filter(|tc| matches!(tc, TestCaseStatus::Accepted | TestCaseStatus::Rejected(_)))
                .count();
            let num_total = status.test_cases.len();

            let running_str = "Running tests".bright_cyan();
            if !status.test_cases.is_empty() {
                print!("\r{running_str} ({num_completed}/{num_total}): {test_cases_str}",);
            } else {
                print!("\r{running_str}...");
            }
        }

        io::stdout().flush().wrap_err("Failed to flush stdout")?;

        if status.verdict.is_finished() {
            println!("\n");

            let num_accepted = status
                .test_cases
                .iter()
                .filter(|tc| matches!(tc, TestCaseStatus::Accepted))
                .count();
            let num_total = status.test_cases.len();

            let outcome = match status.verdict {
                SubmissionStage::Accepted => "ok".bright_green().to_string(),
                SubmissionStage::Rejected(reason) => {
                    format!("{} ({reason})", "failed".bright_red())
                }
                _ => {
                    eyre::bail!("Unreachable state was reached: finished without accepted/rejected")
                }
            };

            let num_passed_str = if !status.test_cases.is_empty() {
                format!(" {num_accepted}/{num_total} passed.")
            } else {
                "".to_string()
            };

            println!(
                "Submission result: {outcome}.{num_passed_str} Time: {}.",
                status.cpu_time
            );

            break;
        }

        sleep(REQUEST_INTERVAL_DURATION).await;
    }

    Ok(())
}

#[derive(Debug)]
struct SubmissionStatus {
    verdict: SubmissionStage,
    test_cases: Vec<TestCaseStatus>,
    cpu_time: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum TestCaseStatus {
    Accepted,
    Rejected(String),
    Unfinished,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum SubmissionStage {
    Accepted,
    Rejected(String),
    BeforeTests(String),
    RunningTests,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
struct SubmissionStatusResponse {
    status_id: u64,
    testcase_index: u64,
    testdata_groups_html: String,
    feedback_html: String,
    judge_feedback_html: String,
    row_html: String,
}

async fn get_submission_status(app: &App, submission_id: &str) -> crate::Result<SubmissionStatus> {
    let submission_url = make_submission_url(app, submission_id)?;

    let res = app
        .client
        .get(submission_url)
        .query(&[("json", "")])
        .send()
        .await
        .wrap_err("Failed to request submission status from Kattis")?;

    let res_body = res
        .json::<SubmissionStatusResponse>()
        .await
        .wrap_err("Failed to read submission status response from Kattis")?;

    SubmissionStatus::from_html(&res_body.row_html)
        .wrap_err("Failed to parse submission status from Kattis")
}

impl SubmissionStatus {
    fn from_html(html_str: &str) -> crate::Result<Self> {
        let html = Html::parse_document(html_str);

        let test_case_statuses: Vec<_> = html
            .select(&Selector::parse(".testcase i.status-icon").unwrap())
            .into_iter()
            .map(|el| TestCaseStatus::from_test_case_icon_html(el.value()))
            .collect();

        let verdict = html
            .select(&Selector::parse("div.status").unwrap())
            .next()
            .map(|el| el.text().collect::<String>())
            .map(|s| SubmissionStage::from_string(&s))
            .ok_or_else(|| eyre::eyre!("Failed to find submission stage"))?;

        // For some reason, the HTML library (html5ever under the hood) doesn't parse the `td`
        // element correctly, so we just extract it ourselves.
        let cpu_regex = Regex::new(
            r#"<td data-type="cpu".*?>(?P<time>[\d\.]+).*?(&nbsp;|\w+)?(?P<unit>\w+)</td>"#,
        )
        .unwrap();
        let cpu_time = cpu_regex
            .captures(html_str)
            .and_then(|c| {
                let time = c.name("time")?.as_str();
                let unit = c.name("unit")?.as_str();
                Some(format!("{time}{unit}"))
            })
            .unwrap_or_else(|| "N/A".to_string());

        Ok(Self {
            verdict,
            test_cases: test_case_statuses,
            cpu_time,
        })
    }
}

impl TestCaseStatus {
    fn from_test_case_icon_html(el: &Element) -> Self {
        if has_class(el, "is-accepted") {
            TestCaseStatus::Accepted
        } else if has_class(el, "is-rejected") {
            let reason = el
                .attr("title")
                .unwrap_or_default()
                .split(':')
                .last()
                .unwrap_or_default()
                .trim();

            TestCaseStatus::Rejected(reason.to_string())
        } else {
            TestCaseStatus::Unfinished
        }
    }
}

impl SubmissionStage {
    fn from_string(stage: &str) -> Self {
        let stage = stage.trim().to_string();

        match stage.to_lowercase().as_str() {
            s if s.contains("running") => Self::RunningTests,
            s if s.contains("new") || s.contains("compiling") => Self::BeforeTests(stage),
            s if s.contains("accepted") => Self::Accepted,
            _ => Self::Rejected(stage),
        }
    }

    fn is_finished(&self) -> bool {
        matches!(self, Self::Accepted | Self::Rejected(_))
    }
}

fn has_class(el: &Element, class: &str) -> bool {
    el.has_class(class, CaseSensitivity::AsciiCaseInsensitive)
}
