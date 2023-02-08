#[derive(Debug)]
pub struct Output {
    pub stdout: String,
    pub stderr: String,
}

pub enum OutputAssertion {
    Contains(String),
    Equals(String),
    Empty,
}

pub enum OutputSource {
    StdOut,
    StdErr,
}

impl Output {
    pub fn assert(&self, source: OutputSource, assertion: OutputAssertion) -> &Self {
        let actual = match source {
            OutputSource::StdOut => &self.stdout,
            OutputSource::StdErr => &self.stderr,
        };

        match assertion {
            OutputAssertion::Contains(expected) => assert_output_contains(actual, expected),
            OutputAssertion::Equals(expected) => assert_output_eq(actual, expected),
            OutputAssertion::Empty => assert_is_empty(actual),
        }

        self
    }
}

fn assert_output_eq(actual: impl AsRef<str>, expected: impl AsRef<str>) {
    assert_eq!(normalise_string(actual), normalise_string(expected));
}

fn assert_output_contains(actual: impl AsRef<str>, expected_substring: impl AsRef<str>) {
    let a = normalise_string(actual);
    let b = normalise_string(expected_substring);

    assert!(
        a.contains(&b),
        "Expected output to contain {b:?} but got {a:?}",
    );
}

fn assert_is_empty(actual: impl AsRef<str>) {
    let actual = normalise_string(actual);

    assert!(
        actual.is_empty(),
        "Expected output to be empty but got {actual:?}"
    );
}

fn normalise_string(s: impl AsRef<str>) -> String {
    let s = s.as_ref();
    let without_ansi_escapes = String::from_utf8(strip_ansi_escapes::strip(s).unwrap()).unwrap();

    without_ansi_escapes
        .trim()
        .lines()
        .map(str::trim)
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn contains(expected: impl Into<String>) -> OutputAssertion {
    OutputAssertion::Contains(expected.into())
}

pub fn equals(expected: impl Into<String>) -> OutputAssertion {
    OutputAssertion::Equals(expected.into())
}
