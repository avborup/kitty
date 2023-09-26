use regex::RegexBuilder;

#[derive(Debug)]
pub struct Output {
    pub stdout: String,
    pub stderr: String,
}

pub enum OutputAssertion {
    Contains(String),
    MatchesRegex(String),
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
            OutputAssertion::MatchesRegex(regex) => assert_output_matches_regex(actual, regex),
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
        "Expected `actual` to contain `expected`:\n  actual:   {a:?}\n  expected: {b:?}",
    );
}

fn assert_output_matches_regex(actual: impl AsRef<str>, regex_str: impl AsRef<str>) {
    let input = normalise_string(actual);

    let regex_str = normalise_string(regex_str)
        .trim()
        .replace("(", r"\(")
        .replace(")", r"\)")
        .replace(".", r"\.");
    let regex = RegexBuilder::new(&regex_str)
        .multi_line(true)
        .dot_matches_new_line(true)
        .build()
        .unwrap();

    assert!(
        regex.is_match(input.as_ref()),
        "Expected `input` to match `regex`:\n  input:   {input:?}\n  regex: {regex_str:?}",
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

/// Check if the output matches a regex in restricted form. Beware that
/// parentheses and periods are automatically escaped.
pub fn matches_regex(regex: impl Into<String>) -> OutputAssertion {
    OutputAssertion::MatchesRegex(regex.into())
}

pub fn equals(expected: impl Into<String>) -> OutputAssertion {
    OutputAssertion::Equals(expected.into())
}
