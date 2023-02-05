use bollard::Docker;
use dockertest::{Composition, DockerTest};
use futures_util::future::BoxFuture;

pub use environment::Environment;
pub use output::{assert_output_eq, Output};

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
    env.exec("mkdir -p /root/.config/kitty").await;
    env.add_file("./kitty.yml", "/root/.config/kitty/kitty.yml");
}
