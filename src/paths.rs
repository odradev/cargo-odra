//! Paths utils.

use std::{
    path::{Path, PathBuf},
    process::Command,
};

use convert_case::{Boundary, Case, Casing};

/// Returns *.wasm filename.
pub fn wasm_file_name(contract_name: &str) -> PathBuf {
    PathBuf::from(contract_name).with_extension("wasm")
}

/// Returns *.wasm file path in target directory.
pub fn wasm_path_in_target(contract_name: &str, project_root: PathBuf) -> PathBuf {
    // extract target dir
    let target_dir = get_build_target_dir();
    project_root
        .join(format!(
            "{}/wasm32-unknown-unknown/release",
            target_dir.display()
        ))
        .join(wasm_file_name(contract_name))
}

fn get_build_target_dir() -> PathBuf {
    let args: Vec<String> = "config get build.target-dir -Z unstable-options"
        .split(' ')
        .map(|s| s.to_string())
        .collect();
    let output = Command::new("cargo").args(args).output();

    if output.is_err() {
        return PathBuf::from("target");
    }

    // convert output to string
    let output_string = String::from_utf8(output.unwrap().stdout);

    if output_string.is_err() {
        return PathBuf::from("target");
    }

    let output_string = output_string.unwrap();

    // output is in format build.target-dir = "../target"
    // convert it to PathBuf
    let target_dir = output_string.split('=').collect::<Vec<&str>>();
    let target_dir = target_dir.get(1);

    if target_dir.is_none() {
        return PathBuf::from("target");
    }

    let target = target_dir.unwrap().trim().to_string().replace('\"', "");

    PathBuf::from(target)
}

/// Returns *.wasm file path in wasm directory.
pub fn wasm_path_in_wasm_dir(contract_name: &str, project_root: &Path) -> PathBuf {
    wasm_dir(project_root).join(wasm_file_name(contract_name))
}

/// Returns wasm directory path.
pub fn wasm_dir(project_root: &Path) -> PathBuf {
    project_root.join("wasm")
}

/// Convert text to a sneak case.
pub fn to_snake_case<T: AsRef<str>>(text: T) -> String {
    text.as_ref()
        .set_boundaries(&Boundary::defaults())
        .remove_boundaries(&[Boundary::UpperDigit, Boundary::LowerDigit])
        .to_case(Case::Snake)
}

/// Convert text to a camel case.
pub fn to_camel_case<T: AsRef<str>>(text: T) -> String {
    text.as_ref()
        .set_boundaries(&Boundary::defaults())
        .remove_boundaries(&[Boundary::UpperDigit, Boundary::LowerDigit])
        .to_case(Case::UpperCamel)
}

/// Turns an arbitrary directory name into a valid Cargo package name.
///
/// A workspace has no package name of its own, so cargo-odra names generated crates after
/// the workspace directory. Cargo only accepts alphanumerics, `-` and `_`, and the name
/// cannot start with a digit; anything else (a clone into `casper-delta.kubaplas.pl`, say)
/// used to produce a manifest Cargo rejects.
pub fn to_crate_name<T: AsRef<str>>(name: T) -> String {
    let mut out = String::new();
    for c in name.as_ref().chars() {
        if c.is_ascii_alphanumeric() || c == '_' {
            out.push(c);
        } else if !out.ends_with('-') {
            out.push('-');
        }
    }
    let out = out.trim_matches('-').to_string();
    match out.chars().next() {
        None => "project".to_string(),
        Some(c) if c.is_ascii_digit() => format!("_{out}"),
        Some(_) => out,
    }
}

#[cfg(test)]
mod tests {
    use super::to_crate_name;

    #[test]
    fn keeps_valid_names_as_they_are() {
        assert_eq!(to_crate_name("casper-delta"), "casper-delta");
        assert_eq!(to_crate_name("my_project"), "my_project");
    }

    #[test]
    fn replaces_characters_cargo_rejects() {
        assert_eq!(
            to_crate_name("casper-delta.kubaplas.pl"),
            "casper-delta-kubaplas-pl"
        );
        assert_eq!(to_crate_name("my project (v2)"), "my-project-v2");
        assert_eq!(to_crate_name("--odd--"), "odd");
    }

    #[test]
    fn never_starts_with_a_digit_or_ends_up_empty() {
        assert_eq!(to_crate_name("2fa-wallet"), "_2fa-wallet");
        assert_eq!(to_crate_name("..."), "project");
    }
}
