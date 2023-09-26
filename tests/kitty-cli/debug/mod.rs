use futures_util::FutureExt;

use crate::helpers::{contains, make_standard_setup, run_with_sandbox, OutputSource::StdErr};

mod answer;
mod input;

#[test]
fn missing_debug_folder_shows_helpful_message() {
    run_with_sandbox(Box::new(|env| {
        async move {
            make_standard_setup(&env).await;

            env.copy("./tests/kitty-cli/data/quadrant", "/work/quadrant");

            env.run("cd quadrant && kitty debug input").await.assert(
                StdErr,
                contains(
                    "Error: You don't have a debug folder. Create it at: /work/quadrant/./debug",
                ),
            );
        }
        .boxed()
    }));
}
