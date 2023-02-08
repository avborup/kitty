use futures_util::FutureExt;
use indoc::indoc;

use crate::helpers::{equals, make_standard_setup, run_with_sandbox, OutputSource::StdOut};

#[test]
fn shows_all_languages() {
    run_with_sandbox(Box::new(|env| {
        async move {
            make_standard_setup(&env).await;

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

            env.run("kitty langs")
                .await
                .assert(StdOut, equals(expected));
        }
        .boxed()
    }));
}

#[test]
fn shows_helpful_when_no_config_is_set() {
    run_with_sandbox(Box::new(|env| {
        async move {
            env.run("kitty langs").await.assert(
                StdOut,
                equals("No languages found. Have you set up your kitty.yml config file?"),
            );
        }
        .boxed()
    }));
}
