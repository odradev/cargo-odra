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
    let cmd = "cargo config get build.target-dir -Z unstable-options";
    let output = Command::new("sh").arg("-c").arg(cmd).output();

    if output.is_err() {
        return PathBuf::from("target");
    }

    // convert output to string
    let target_dir = String::from_utf8(output.unwrap().stdout).unwrap();

    // output is in format build.target-dir = "../target"
    // convert it to PathBuf
    let target_dir = target_dir.split('=').collect::<Vec<&str>>();
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
        .with_boundaries(&Boundary::defaults())
        .without_boundaries(&[Boundary::UpperDigit, Boundary::LowerDigit])
        .to_case(Case::Snake)
}
