//! Paths utils.

use std::path::PathBuf;

use convert_case::{Boundary, Case, Casing};

/// Returns *.wasm filename.
pub fn wasm_file_name(contract_name: &str) -> PathBuf {
    PathBuf::from(contract_name).with_extension("wasm")
}

/// Returns *.wasm file path in target directory.
pub fn wasm_path_in_target(contract_name: &str, project_root: PathBuf) -> PathBuf {
    project_root
        .join("target/wasm32-unknown-unknown/release")
        .join(wasm_file_name(contract_name))
}

/// Returns *.wasm file path in wasm directory.
pub fn wasm_path_in_wasm_dir(contract_name: &str, project_root: PathBuf) -> PathBuf {
    wasm_dir(project_root).join(wasm_file_name(contract_name))
}

/// Returns wasm directory path.
pub fn wasm_dir(project_root: PathBuf) -> PathBuf {
    project_root.join("wasm")
}

/// Convert text to a sneak case.
pub fn to_snake_case<T: AsRef<str>>(text: T) -> String {
    text.as_ref()
        .with_boundaries(&Boundary::defaults())
        .without_boundaries(&[Boundary::UpperDigit, Boundary::LowerDigit])
        .to_case(Case::Snake)
}

pub fn to_snake_titlecase<T: AsRef<str>>(text: T) -> String {
    let mut text = to_snake_case(text);
    if let Some(r) = text.get_mut(0..1) {
        r.make_ascii_uppercase();
    }

    text
}
