use std::env;

use bollard::Docker;
use dockertest::{Composition, DockerTest};
use dotenv::dotenv;
use futures_util::future::BoxFuture;

pub use environment::Environment;
pub use output::*;

mod environment;
mod output;

pub fn run_with_sandbox(to_run: Box<dyn Fn(Environment) -> BoxFuture<'_, ()> + Send + Sync>) {
    let docker = Docker::connect_with_socket_defaults().unwrap();
    let mut test = DockerTest::new();

    let comp = Composition::with_repository("kitty-test")
        // This is so jank. How to fix? :/
        // The problem is that if I don't do this, the container will exit
        // immediately, causing the tests to fail.
        .with_cmd(vec!["sleep".to_string(), "120".to_string()]);

    test.add_composition(comp);

    test.run(|ops| async move {
        let container = ops.handle("kitty-test");

        let env = Environment { docker, container };

        to_run(env).await;
    });
}

pub async fn make_standard_setup(env: &Environment<'_>) {
    // Must call .ok(), not unwrap, since an error is returned if no .env exists
    dotenv().ok();

    env.run("mkdir -p /root/.config/kitty").await;
    env.add_file("./kitty.yml", "/root/.config/kitty/kitty.yml");

    if let Ok(token) = env::var("KATTIS_TEST_TOKEN") {
        let kattisrc = format!(
            indoc::indoc! {"
                [user]
                username: kitty-tester
                token: {}

                [kattis]
                hostname: open.kattis.com
                loginurl: https://open.kattis.com/login
                submissionurl: https://open.kattis.com/submit
                submissionsurl: https://open.kattis.com/submissions
            "},
            token
        );

        env.run(&format!(
            "echo '{kattisrc}' > /root/.config/kitty/.kattisrc"
        ))
        .await;
    }
}

pub async fn add_template(env: &Environment<'_>, template_file: &str, template_contents: &str) {
    env.run("mkdir -p /root/.config/kitty/templates").await;
    env.run(&format!(
        "echo '{template_contents}' > /root/.config/kitty/templates/{template_file}"
    ))
    .await;
}
