use std::path::Path;

pub fn path_to_str(path: &Path) -> String {
    let path_str = path.to_str().expect("path contained invalid unicode").to_string();

    #[cfg(windows)]
    let path_str = path_str.replace(r"\", r"\\");

    path_str
}
