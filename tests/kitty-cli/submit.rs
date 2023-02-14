use futures_util::FutureExt;
use serial_test::serial;

use crate::helpers::{contains, make_standard_setup, run_with_sandbox, OutputSource::StdOut};

#[test]
#[serial]
fn correct_answer_shows_complete_test_run() {
    run_with_sandbox(Box::new(|env| {
        async move {
            make_standard_setup(&env).await;

            env.copy("./tests/kitty-cli/data/quadrant", "/work/quadrant");

            env.run("kitty submit quadrant -y")
                .await
                .assert(
                    StdOut,
                    contains(indoc::indoc! {r#"
                        Problem:  quadrant
                        Language: Python 3
                        File:     quadrant.py
                        Submitted solution to https://open.kattis.com/submissions
                    "#}),
                )
                .assert(
                    StdOut,
                    contains("Running tests (17/17): 🟢🟢🟢🟢🟢🟢🟢🟢🟢🟢🟢🟢🟢🟢🟢🟢🟢"),
                )
                .assert(
                    StdOut,
                    contains("Submission result: ok. 17/17 passed. Time: 0.0"),
                );
        }
        .boxed()
    }));
}

#[test]
#[serial]
fn wrong_answer_shows_incomplete_test_run() {
    run_with_sandbox(Box::new(|env| {
        async move {
            make_standard_setup(&env).await;

            env.copy("./tests/kitty-cli/data/quadrant", "/work/quadrant");
            env.copy(
                "./tests/kitty-cli/data/quadrant-wrong-answer-submit.py",
                "/work/quadrant/quadrant.py",
            );

            env.run("kitty submit quadrant -y")
                .await
                .assert(
                    StdOut,
                    contains(indoc::indoc! {r#"
                        Problem:  quadrant
                        Language: Python 3
                        File:     quadrant.py
                        Submitted solution to https://open.kattis.com/submissions
                    "#}),
                )
                .assert(
                    StdOut,
                    contains("Running tests (4/17): 🟢🟢🟢🔴⚪⚪⚪⚪⚪⚪⚪⚪⚪⚪⚪⚪⚪"),
                )
                .assert(
                    StdOut,
                    contains("Submission result: failed (Wrong Answer). 3/17 passed. Time: 0.0"),
                );
        }
        .boxed()
    }));
}

#[test]
#[serial]
fn runtime_error_shows_incomplete_test_run() {
    run_with_sandbox(Box::new(|env| {
        async move {
            make_standard_setup(&env).await;

            env.copy("./tests/kitty-cli/data/quadrant", "/work/quadrant");
            env.copy(
                "./tests/kitty-cli/data/quadrant-runtime-error.py",
                "/work/quadrant/quadrant.py",
            );

            env.run("kitty submit quadrant -y")
                .await
                .assert(
                    StdOut,
                    contains(indoc::indoc! {r#"
                        Problem:  quadrant
                        Language: Python 3
                        File:     quadrant.py
                        Submitted solution to https://open.kattis.com/submissions
                    "#}),
                )
                .assert(
                    StdOut,
                    contains("Running tests (1/17): 🔴⚪⚪⚪⚪⚪⚪⚪⚪⚪⚪⚪⚪⚪⚪⚪⚪"),
                )
                .assert(
                    StdOut,
                    contains("Submission result: failed (Run-Time Error). 0/17 passed. Time: 0.0"),
                );
        }
        .boxed()
    }));
}
