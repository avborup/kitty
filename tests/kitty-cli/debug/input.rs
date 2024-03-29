use futures_util::FutureExt;

use crate::helpers::{
    contains, make_standard_setup, matches_regex, run_with_sandbox,
    OutputSource::{StdErr, StdOut},
};

#[test]
fn input_generator_finishes_on_valid_solution() {
    run_with_sandbox(Box::new(|env| {
        async move {
            make_standard_setup(&env).await;

            env.copy("./tests/kitty-cli/data/quadrant", "/work/quadrant");
            env.run("cd /work/quadrant && mkdir debug").await;
            env.copy(
                "./tests/kitty-cli/data/generators/quadrant-input-generator.py",
                "/work/quadrant/debug/input.py",
            );

            env.run("cd quadrant && kitty debug input")
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
fn input_generator_shows_error_if_solution_crashes() {
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

            env.run("cd quadrant && kitty debug input").await.assert(
                StdOut,
                matches_regex(indoc::indoc! {r#"
                    Running test \d+/\d+... ❌

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
fn missing_input_generator_shows_helpful_message() {
    run_with_sandbox(Box::new(|env| {
        async move {
            make_standard_setup(&env).await;

            env.copy("./tests/kitty-cli/data/quadrant", "/work/quadrant");
            env.run("cd /work/quadrant && mkdir debug").await;

            env.run("cd quadrant && kitty debug input")
                .await
                .assert(
                    StdErr,
                    contains("Error: No input generator file found in the debug folder: /work/quadrant/./debug. See the help message for how to create one."),
                );
        }
        .boxed()
    }));
}

#[test]
fn input_generator_shows_error_if_generator_fails() {
    run_with_sandbox(Box::new(|env| {
        async move {
            make_standard_setup(&env).await;

            env.copy("./tests/kitty-cli/data/quadrant", "/work/quadrant");
            env.run("cd /work/quadrant && mkdir debug").await;
            env.copy(
                "./tests/kitty-cli/data/generators/quadrant-buggy-input-generator.py",
                "/work/quadrant/debug/input.py",
            );

            env.run("cd quadrant && kitty debug input")
                .await
                .assert(StdOut, contains("Running test 1/100... ❌"))
                .assert(
                    StdErr,
                    contains(indoc::indoc! {r#"
                        Error: Your input generator exited with a non-zero exit code (exit status: 1).

                        Generator output:
                        42
                        Traceback (most recent call last):
                          File "/work/quadrant/./debug/input.py", line 2, in <module>
                            raise Exception("Buggy input generator")
                        Exception: Buggy input generator
                    "#}),
                );
        }
        .boxed()
    }));
}
