# Kitty contribution guidelines

## Dependencies

For compiling and running kitty, you will need:

- [Rust](https://www.rust-lang.org/tools/install)

For running tests you will need:

- [Docker](https://www.docker.com/)

## Build and run

Build:

```sh
cargo build
```

Run:

```sh
cargo run -- <args for kitty here>
```

## Testing

### Environment variables

To run the test suite, you must provide a username and a authentication token for Kattis which will be used to fetch and submit test samples. The test suite expects the following environment variables:

- `KATTIS_TEST_USERNAME`: your test account's Kattis username
- `KATTIS_TEST_TOKEN`: your test account's Kattis authentication token

You can do this in two ways:

1. Set the environment variables in your shell
2. Create a `.env` file in the project root that looks like the below. The test suite will load it if present.
    ```env
    KATTIS_TEST_USERNAME=<insert username>
    KATTIS_TEST_TOKEN=<insert token>
    ```

### Run tests

To run tests:

```sh
make test
```

If you don't have `make` (maybe you're on Windows), just look inside the `Makefile` and run the commands directly.

The test recipe builds a Docker image in which it installs kitty from your local version of the project, and then it runs `cargo test`. The tests will each run in their own Docker container to fully sandbox them.

Please note that if you change kitty's source code (anything inside `src`), you have to rebuild the Docker image. `make test` does this for you.

## Linting

All source code is formatted with `rustfmt` and linted with `clippy`. If you make a pull request, checks for these will be run automatically.
