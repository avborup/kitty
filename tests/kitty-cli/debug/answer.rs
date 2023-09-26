use futures_util::FutureExt;

use crate::helpers::{
    contains, make_standard_setup, matches_regex, run_with_sandbox,
    OutputSource::{StdErr, StdOut},
};

#[test]
fn answer_generator_finishes_on_valid_solution() {
    run_with_sandbox(Box::new(|env| {
        async move {
            make_standard_setup(&env).await;

            env.copy("./tests/kitty-cli/data/quadrant", "/work/quadrant");
            env.run("cd /work/quadrant && mkdir debug").await;
            env.copy(
                "./tests/kitty-cli/data/generators/quadrant-input-generator.py",
                "/work/quadrant/debug/input.py",
            );
            env.copy(
                "./tests/kitty-cli/data/quadrant.c",
                "/work/quadrant/debug/answer.c",
            );

            env.run("cd quadrant && kitty debug answer")
                .await
                .assert(StdOut, contains("Running test 1/100..."))
                .assert(
                    StdOut,
                    matches_regex(indoc::indoc! {r#"
                        Running test 100/100... ✅

                        Passed all 100 test cases. Running times: min \d+.\d+s, max \d+.\d+s, average \d+.\d+s.
                    "#}),
                );
        }
        .boxed()
    }));
}

#[test]
fn missing_answer_generator_shows_helpful_message() {
    run_with_sandbox(Box::new(|env| {
        async move {
            make_standard_setup(&env).await;

            env.copy("./tests/kitty-cli/data/quadrant", "/work/quadrant");
            env.run("cd /work/quadrant && mkdir debug").await;
            env.copy(
                "./tests/kitty-cli/data/generators/quadrant-input-generator.py",
                "/work/quadrant/debug/input.py",
            );

            env.run("cd quadrant && kitty debug answer")
                .await
                .assert(
                    StdErr,
                    contains("Error: No answer generator file found in the debug folder: /work/quadrant/./debug. See the help message for how to create one."),
                );
        }
        .boxed()
    }));
}

#[test]
fn answer_validator_wrong_answer_shows_actual_output_and_input_parameters() {
    run_with_sandbox(Box::new(|env| {
        async move {
            make_standard_setup(&env).await;

            env.copy("./tests/kitty-cli/data/quadrant", "/work/quadrant");
            env.copy(
                "./tests/kitty-cli/data/quadrant-wrong-answer.py",
                "/work/quadrant/quadrant.py",
            );
            env.run("cd /work/quadrant && mkdir debug").await;
            env.copy(
                "./tests/kitty-cli/data/generators/quadrant-input-generator.py",
                "/work/quadrant/debug/input.py",
            );
            env.copy(
                "./tests/kitty-cli/data/quadrant.c",
                "/work/quadrant/debug/answer.c",
            );

            env.run("cd quadrant && kitty debug answer").await.assert(
                StdOut,
                matches_regex(indoc::indoc! {r#"
                    Running test \d+/100... ❌

                    Expected:
                    \d

                    Actual:
                    \d

                    Input:
                    -?\d+
                    -?\d+

                    Saving input to \d+-wrong-answer.in
                    Saving expected answer to \d+-wrong-answer.ans
                    Saving your answer to \d+-wrong-answer.output

                    To use these as part of normal `kitty test` runs, move the .in/.ans files to the test folder of your solution.

                    The saved files can be found in /work/quadrant/./debug/saved
                "#}),
            );
        }
        .boxed()
    }));
}

#[test]
fn answer_validator_runtime_error_shows_error_and_input_parameters() {
    run_with_sandbox(Box::new(|env| {
        async move {
            make_standard_setup(&env).await;

            env.copy("./tests/kitty-cli/data/quadrant", "/work/quadrant");
            env.copy(
                "./tests/kitty-cli/data/quadrant-runtime-error.py",
                "/work/quadrant/quadrant.py",
            );
            env.run("cd /work/quadrant && mkdir debug").await;
            env.copy(
                "./tests/kitty-cli/data/generators/quadrant-input-generator.py",
                "/work/quadrant/debug/input.py",
            );
            env.copy(
                "./tests/kitty-cli/data/quadrant.c",
                "/work/quadrant/debug/answer.c",
            );

            env.run("cd quadrant && kitty debug answer").await.assert(
                StdOut,
                matches_regex(indoc::indoc! {r#"
                    Running test \d+/100... ❌

                    Runtime error:

                    Traceback (most recent call last):
                      File "/work/quadrant/./quadrant.py", line 6, in <module>
                        raise Exception("I don't know what quadrant this is in!")
                    Exception: I don't know what quadrant this is in!

                    Input:
                    -?\d+
                    -?\d+

                    Saving input to \d+-runtime-error.in
                    Saving your solution's output to \d+-runtime-error.output

                    The saved files can be found in /work/quadrant/./debug/saved
                "#}),
            );
        }
        .boxed()
    }));
}

#[test]
fn buggy_answer_validator_shows_runtime_error() {
    run_with_sandbox(Box::new(|env| {
        async move {
            make_standard_setup(&env).await;

            env.copy("./tests/kitty-cli/data/quadrant", "/work/quadrant");
            env.run("cd /work/quadrant && mkdir debug").await;
            env.copy(
                "./tests/kitty-cli/data/generators/quadrant-input-generator.py",
                "/work/quadrant/debug/input.py",
            );
            env.copy(
                "./tests/kitty-cli/data/quadrant-runtime-error.py",
                "/work/quadrant/debug/answer.py",
            );

            env.run("cd quadrant && kitty debug answer")
                .await
                .assert(StdOut, matches_regex(r"Running test \d+/100... ❌"))
                .assert(
                    StdErr,
                    contains(indoc::indoc! {r#"
                        Error: Your answer generator exited with a non-zero exit code (exit status: 1).

                        Generator output:

                        Traceback (most recent call last):
                          File "/work/quadrant/./debug/answer.py", line 6, in <module>
                            raise Exception("I don't know what quadrant this is in!")
                        Exception: I don't know what quadrant this is in!
                    "#}),
                );
        }
        .boxed()
    }));
}
