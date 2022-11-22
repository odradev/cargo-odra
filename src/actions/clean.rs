//! Module responsible for cleaning Odra projects.

use crate::{command, paths};

/// Removes wasm folder, .builder* folders and runs `cargo clean`.
pub fn clean_action() {
    for folder in glob::glob("wasm").unwrap().flatten() {
        command::rm_dir(folder);
    }

    for folder in glob::glob(".builder*").unwrap().flatten() {
        command::rm_dir(folder);
    }

    command::cargo_clean(paths::project_dir());
}
