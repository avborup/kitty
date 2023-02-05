use futures_util::FutureExt;
use indoc::indoc;

use crate::helpers::{assert_output_eq, make_standard_setup, run_with_sandbox};

#[test]
fn langs_shows_all_languages() {
    run_with_sandbox(Box::new(|env| {
        async move {
            make_standard_setup(&env).await;

            let output = env.exec("kitty langs").await;

            let expected = indoc! {"
                Name       Extension
                C          c
                C#         cs
                C++        cpp
                Go         go
                Haskell    hs
                Java       java
                Python 3   py
                Rust       rs
            "};

            assert_output_eq(output.stdout, expected);
        }
        .boxed()
    }));
}

#[test]
fn langs_shows_helpful_when_no_config_is_set() {
    run_with_sandbox(Box::new(|env| {
        async move {
            let output = env.exec("kitty langs").await;

            assert_output_eq(
                output.stdout,
                "No languages found. Have you set up your kitty.yml config file?",
            );
        }
        .boxed()
    }));
}