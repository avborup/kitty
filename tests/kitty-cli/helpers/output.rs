#[derive(Debug)]
pub struct Output {
    pub stdout: String,
    pub stderr: String,
}

pub fn assert_output_eq(actual: impl AsRef<str>, expected: impl AsRef<str>) {
    assert_eq!(normalise_string(actual), normalise_string(expected),);
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
