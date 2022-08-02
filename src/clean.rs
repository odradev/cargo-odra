use std::os::unix::prelude::CommandExt;
use std::path::PathBuf;
use std::process::Command;

pub struct Clean {}

impl Clean {
    pub fn new() -> Clean {
        Clean {}
    }

    pub fn clean(&self) {
        // TODO: Usunąć wasm
        for folder in glob::glob(".backend*").unwrap().flatten() {
            Clean::rm_rf(folder);
        }

        for folder in glob::glob(".builder*").unwrap().flatten() {
            Clean::rm_rf(folder);
        }

        println!("Running cargo clean...");
        Command::new("cargo").args(["clean"]).exec();
    }

    fn rm_rf(folder: PathBuf) {
        rm_rf::ensure_removed(folder.clone())
            .unwrap_or_else(|_| panic!("Couldn't remove {}", folder.display()));
        println!("Removing {}...", folder.display());
    }
}
