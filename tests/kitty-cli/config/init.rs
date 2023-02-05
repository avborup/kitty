use futures_util::FutureExt;

use crate::helpers::{assert_output_eq, run_with_sandbox};

#[test]
fn init_creates_folders() {
    run_with_sandbox(Box::new(|env| {
        async move {
            assert_output_eq(env.exec("ls /root/.config").await.stdout, "");

            let output = env.exec("kitty config init").await;

            assert!(
                output.stdout.contains("Initialised config directory"),
                "Stdout did not contain success text: {:#?}",
                output
            );

            assert_output_eq(env.exec("ls /root/.config").await.stdout, "kitty");
        }
        .boxed()
    }));
}
