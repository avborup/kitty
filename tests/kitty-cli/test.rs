use futures_util::FutureExt;

use crate::helpers::{
    contains, make_standard_setup, run_with_sandbox,
    OutputSource::{StdErr, StdOut},
};

#[test]
fn correct_solution_passes() {
    run_with_sandbox(Box::new(|env| {
        async move {
            make_standard_setup(&env).await;

            env.copy("./tests/kitty-cli/data/quadrant", "/work/quadrant");

            env.run("cd quadrant && kitty test").await.assert(
                StdOut,
                contains(indoc::indoc! {r#"
                    Running 2 tests

                    test 1 ... ✅
                    test 2 ... ✅

                    Test result: ok. 2 passed; 0 failed.
                "#}),
            );
        }
        .boxed()
    }));
}

#[test]
fn wrong_answer_is_shown() {
    run_with_sandbox(Box::new(|env| {
        async move {
            make_standard_setup(&env).await;

            env.copy("./tests/kitty-cli/data/quadrant", "/work/quadrant");
            env.copy(
                "./tests/kitty-cli/data/quadrant-wrong-answer.py",
                "/work/quadrant/quadrant.py",
            );

            env.run("cd quadrant && kitty test").await.assert(
                StdOut,
                contains(indoc::indoc! {r#"
                    Running 2 tests

                    test 1 ... ❌
                    Expected:
                    1

                    Actual:
                    3

                    test 2 ... ✅

                    Test result: failed. 1 passed; 1 failed.
                "#}),
            );
        }
        .boxed()
    }));
}

#[test]
fn runtime_error_is_shown() {
    run_with_sandbox(Box::new(|env| {
        async move {
            make_standard_setup(&env).await;

            env.copy("./tests/kitty-cli/data/quadrant", "/work/quadrant");
            env.copy(
                "./tests/kitty-cli/data/quadrant-runtime-error.py",
                "/work/quadrant/quadrant.py",
            );

            env.run("cd quadrant && kitty test").await.assert(
                StdOut,
                contains(indoc::indoc! {r#"
                    Running 2 tests

                    test 1 ... ❌
                    Runtime error:
                    Traceback (most recent call last):
                      File "/work/quadrant/./quadrant.py", line 10, in <module>
                        raise Exception("I don't know what quadrant this is in!")
                    Exception: I don't know what quadrant this is in!


                    test 2 ... ✅

                    Test result: failed. 1 passed; 1 failed.
                "#}),
            );
        }
        .boxed()
    }));
}

#[test]
fn can_compile_and_run() {
    run_with_sandbox(Box::new(|env| {
        async move {
            make_standard_setup(&env).await;

            env.copy("./tests/kitty-cli/data/quadrant", "/work/quadrant");
            env.copy("./tests/kitty-cli/data/quadrant.c", "/work/quadrant");

            env.run("cd quadrant && kitty test -f quadrant.c")
                .await
                .assert(
                    StdOut,
                    contains(indoc::indoc! {r#"
                        Running 2 tests

                        test 1 ... ✅
                        test 2 ... ✅

                        Test result: ok. 2 passed; 0 failed.
                    "#}),
                );
        }
        .boxed()
    }));
}

#[test]
fn compile_error_is_shown() {
    run_with_sandbox(Box::new(|env| {
        async move {
            make_standard_setup(&env).await;

            env.copy("./tests/kitty-cli/data/quadrant", "/work/quadrant");
            env.copy("./tests/kitty-cli/data/quadrant-compile-error.c", "/work/quadrant");

            env.run("cd quadrant && kitty test -f quadrant-compile-error.c")
                .await
                .assert(
                    StdErr,
                    contains(indoc::indoc! {r#"
                        Compilation error:

                        /work/quadrant/quadrant-compile-error.c: In function 'main':
                        /work/quadrant/quadrant-compile-error.c:4:27: error: 'qrd' undeclared (first use in this function)
                            4 |     while (scanf("%lld", &qrd) == 2) {
                              |                           ^~~
                        /work/quadrant/quadrant-compile-error.c:4:27: note: each undeclared identifier is reported only once for each function it appears in

                        Error: Failed to compile program (exit status: 1)
                    "#}),
                );
        }
        .boxed()
    }));
}

#[test]
fn language_override_works() {
    run_with_sandbox(Box::new(|env| {
        async move {
            make_standard_setup(&env).await;

            env.copy("./tests/kitty-cli/data/quadrant", "/work/quadrant");
            env.run("mv quadrant/quadrant.py quadrant/quadrant.c").await;

            env.run("cd quadrant && kitty test --lang py").await.assert(
                StdOut,
                contains(indoc::indoc! {r#"
                    Running 2 tests

                    test 1 ... ✅
                    test 2 ... ✅

                    Test result: ok. 2 passed; 0 failed.
                "#}),
            );
        }
        .boxed()
    }));
}

#[test]
fn filter_only_runs_matching_tests() {
    run_with_sandbox(Box::new(|env| {
        async move {
            make_standard_setup(&env).await;

            env.copy("./tests/kitty-cli/data/quadrant", "/work/quadrant");

            env.run("cd quadrant/test && cp 1.in custom01.in").await;
            env.run("cd quadrant/test && cp 1.ans custom01.ans").await;
            env.run("cd quadrant/test && cp 2.in custom02.in").await;
            env.run("cd quadrant/test && cp 2.ans custom02.ans").await;

            env.run("cd quadrant && kitty test --filter 1")
                .await
                .assert(
                    StdOut,
                    contains(indoc::indoc! {r#"
                        Running 2 tests

                        test 1 ... ✅
                        test custom01 ... ✅

                        Test result: ok. 2 passed; 0 failed.
                    "#}),
                );

            env.run("cd quadrant && kitty test --filter '^custom'")
                .await
                .assert(
                    StdOut,
                    contains(indoc::indoc! {r#"
                        Running 2 tests

                        test custom01 ... ✅
                        test custom02 ... ✅

                        Test result: ok. 2 passed; 0 failed.
                    "#}),
                );
        }
        .boxed()
    }));
}
