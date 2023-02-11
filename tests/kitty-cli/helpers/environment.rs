use std::{path::Path, process::Command};

use bollard::{
    container::LogOutput,
    exec::{CreateExecOptions, StartExecResults},
    Docker,
};
use dockertest::RunningContainer;
use futures_util::StreamExt;

use super::Output;

pub struct Environment<'a> {
    pub(super) docker: Docker,
    pub(super) container: &'a RunningContainer,
}

impl<'a> Environment<'a> {
    pub async fn run(&self, cmd: &str) -> Output {
        let exec = self
            .docker
            .create_exec(
                &self.container.name(),
                CreateExecOptions {
                    cmd: Some(vec!["sh", "-c", cmd]),
                    attach_stdout: Some(true),
                    attach_stderr: Some(true),
                    ..Default::default()
                },
            )
            .await
            .unwrap();

        let mut stdout = String::new();
        let mut stderr = String::new();

        fn bytes_to_str(bytes: impl Into<Vec<u8>>) -> String {
            String::from_utf8(bytes.into()).expect("Invalid UTF-8 in output")
        }

        if let StartExecResults::Attached { mut output, .. } =
            self.docker.start_exec(&exec.id, None).await.unwrap()
        {
            while let Some(Ok(msg)) = output.next().await {
                match msg {
                    LogOutput::StdOut { message } => stdout.push_str(&bytes_to_str(message)),
                    LogOutput::StdErr { message } => stderr.push_str(&bytes_to_str(message)),
                    other => panic!("Unexpected output from command: {:#?}", other),
                }
            }
        } else {
            unreachable!();
        }

        let output = Output { stdout, stderr };

        dbg!((cmd, &output));

        output
    }

    pub fn copy(&self, local_file: impl AsRef<Path>, destination_in_container: impl AsRef<Path>) {
        let local_file_path = local_file.as_ref().to_string_lossy();
        let dest_file_path = destination_in_container.as_ref().to_string_lossy();
        let container_name = self.container.name();

        // Docs on `docker cp`: https://docs.docker.com/engine/reference/commandline/cp/#examples
        let output = Command::new("docker")
            .args([
                "cp",
                local_file_path.as_ref(),
                &format!("{container_name}:{dest_file_path}"),
            ])
            .output()
            .unwrap();

        if !output.status.success() {
            panic!("Failed to copy file to container: {:?}", &output);
        }
    }
}
