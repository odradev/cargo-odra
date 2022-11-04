//! Module responsible for cleaning Odra projects
use std::os::unix::prelude::CommandExt;
use std::path::PathBuf;
use std::process::Command;

use crate::log;

/// Removes .builder* folders and runs `cargo clean`
pub fn clean_action() {
    for folder in glob::glob("wasm").unwrap().flatten() {
        rm_rf(folder);
    }

    for folder in glob::glob(".builder*").unwrap().flatten() {
        rm_rf(folder);
    }

    log::info("Running cargo clean...");
    Command::new("cargo").args(["clean"]).exec();
}

fn rm_rf(folder: PathBuf) {
    rm_rf::ensure_removed(folder.clone())
        .unwrap_or_else(|_| panic!("Couldn't remove {}", folder.display()));
    log::info(format!("Removing {}...", folder.display()));
}
