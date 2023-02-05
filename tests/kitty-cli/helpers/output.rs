#[derive(Debug)]
pub struct Output {
    pub stdout: String,
    pub stderr: String,
}

pub fn assert_output_eq(actual: impl AsRef<str>, expected: impl AsRef<str>) {
    let actual = actual.as_ref();
    let expected = expected.as_ref();

    assert_eq!(normalise_string(actual), normalise_string(expected),);
}

fn normalise_string(s: impl AsRef<str>) -> String {
    s.as_ref()
        .trim()
        .lines()
        .map(str::trim)
        .collect::<Vec<_>>()
        .join("\n")
}
