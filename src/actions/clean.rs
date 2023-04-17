//! Module responsible for cleaning Odra projects.

use std::path::PathBuf;

use crate::command;

/// Removes wasm folder, .builder* folders and runs `cargo clean`.
pub fn clean_action(project_root: PathBuf) {
    for folder in glob::glob(project_root.join("wasm/*").as_os_str().to_str().unwrap())
        .unwrap()
        .flatten()
    {
        command::rm_dir(folder);
    }

    for folder in glob::glob(project_root.join(".builder*").as_os_str().to_str().unwrap())
        .unwrap()
        .flatten()
    {
        command::rm_dir(folder);
    }

    command::cargo_clean(project_root);
}
