//! Functions used to read and manage Cargo.toml of a project using odra
use std::process;

use cargo_toml::{Dependency, Manifest};
use prettycli::critical;

/// Returns Dependency of Odra, taken from project's Cargo.toml
pub fn odra_dependency() -> Dependency {
    let cargo_toml = match Manifest::from_path("Cargo.toml") {
        Ok(manifest) => manifest,
        Err(err) => {
            critical(&format!("Failed to read Cargo.toml: {}", err));
            process::exit(2);
        }
    };

    cargo_toml.dependencies.get("odra").unwrap().clone()
}
