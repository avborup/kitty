use futures_util::FutureExt;

use crate::helpers::{
    add_template, contains, equals, make_standard_setup, run_with_sandbox,
    OutputAssertion::Empty,
    OutputSource::{StdErr, StdOut},
};

#[test]
fn creates_folders_and_downloads_tests() {
    run_with_sandbox(Box::new(|env| {
        async move {
            make_standard_setup(&env).await;

            env.run("ls").await.assert(StdOut, Empty);

            env.run("kitty get quadrant")
                .await
                .assert(StdOut, contains("Created solution folder for quadrant"));

            env.run("ls").await.assert(StdOut, contains("quadrant"));

            let test_files_output = env.run("ls quadrant/test").await;

            ["1.in", "1.ans", "2.in", "2.ans"]
                .into_iter()
                .for_each(|s| {
                    test_files_output.assert(StdOut, contains(s));
                });

            env.run("cat quadrant/test/2.in")
                .await
                .assert(StdOut, equals("9\n-13"));
            env.run("cat quadrant/test/2.ans")
                .await
                .assert(StdOut, equals("4"));
        }
        .boxed()
    }));
}

#[test]
fn reports_if_problem_does_not_exist() {
    run_with_sandbox(Box::new(|env| {
        async move {
            make_standard_setup(&env).await;

            env.run("kitty get idontexist")
                .await
                .assert(StdErr, contains("'idontexist' does not exist"));
        }
        .boxed()
    }));
}

#[test]
fn copies_template() {
    run_with_sandbox(Box::new(|env| {
        async move {
            make_standard_setup(&env).await;

            add_template(
                &env,
                "template.java",
                indoc::indoc! {"
                    import java.util.Scanner;

                    public class $FILENAME {
                        public static void main(String[] args) {
                            Scanner sc = new Scanner(System.in);

                            sc.close();
                        }
                    }
                "},
            )
            .await;

            env.run("kitty get quadrant --lang java").await;

            env.run("cat quadrant/quadrant.java")
                .await
                .assert(StdOut, contains("public class quadrant {"));
        }
        .boxed()
    }));
}
