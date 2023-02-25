use futures_util::FutureExt;

use crate::helpers::{
    contains, equals, run_with_sandbox, OutputAssertion::Empty, OutputSource::StdOut,
};

#[test]
fn creates_folders() {
    run_with_sandbox(Box::new(|env| {
        async move {
            env.run("ls /root/.config").await.assert(StdOut, Empty);

            env.run("kitty config init")
                .await
                .assert(StdOut, contains("Initialised config directory"));

            env.run("ls /root/.config")
                .await
                .assert(StdOut, equals("kitty"));
        }
        .boxed()
    }));
}
