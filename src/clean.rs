//! Module responsible for cleaning Odra projects
use crate::info;
use std::os::unix::prelude::CommandExt;
use std::path::PathBuf;
use std::process::Command;

/// Struct holding code of Clean module.
pub struct Clean {}

impl Clean {
    /// Creates new Clean instance
    pub fn new() -> Clean {
        Clean {}
    }

    /// Removes .builder* folders and runs `cargo clean`
    pub fn clean(&self) {
        for folder in glob::glob("wasm").unwrap().flatten() {
            Clean::rm_rf(folder);
        }

        for folder in glob::glob(".builder*").unwrap().flatten() {
            Clean::rm_rf(folder);
        }

        info("Running cargo clean...");
        Command::new("cargo").args(["clean"]).exec();
    }

    fn rm_rf(folder: PathBuf) {
        rm_rf::ensure_removed(folder.clone())
            .unwrap_or_else(|_| panic!("Couldn't remove {}", folder.display()));
        info(&format!("Removing {}...", folder.display()));
    }
}
