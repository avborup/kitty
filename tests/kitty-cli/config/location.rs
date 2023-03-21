use futures_util::FutureExt;

use crate::helpers::{equals, run_with_sandbox, OutputSource::StdOut};

#[test]
fn config_dir_is_overridden() {
    run_with_sandbox(Box::new(|env| {
        async move {
            env.run("KATTIS_KITTY_CONFIG_DIR=/root/kitty-config-dir kitty config location")
                .await
                .assert(
                    StdOut,
                    equals(indoc::indoc! {r#"
                        Your config files should go in this directory:

                            /root/kitty-config-dir

                        More specifically:
                         - Your .kattisrc file:   /root/kitty-config-dir/.kattisrc
                         - Your kitty.yml file:   /root/kitty-config-dir/kitty.yml
                         - Your templates folder: /root/kitty-config-dir/templates
                    "#}),
                );
        }
        .boxed()
    }));
}
