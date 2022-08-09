/// Functions used to read and manage Cargo.toml of a project using odra
use std::process;

use cargo_toml::{DependencyDetail, Manifest};
use prettycli::critical;

pub fn odra_details() -> Option<DependencyDetail> {
    let cargo_toml = match Manifest::from_path("Cargo.toml") {
        Ok(manifest) => manifest,
        Err(_) => {
            critical("Cargo.toml not found, exiting.");
            process::exit(2);
        }
    };

    cargo_toml
        .dependencies
        .get("odra")
        .unwrap()
        .detail()
        .cloned()
}
