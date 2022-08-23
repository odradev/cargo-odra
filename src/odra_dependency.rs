//! Functions used to read and manage Cargo.toml of a project using odra
use crate::errors::Error;
use cargo_toml::{Dependency, Manifest};

/// Returns Dependency of Odra, taken from project's Cargo.toml
pub fn odra_dependency() -> Dependency {
    let cargo_toml = match Manifest::from_path("Cargo.toml") {
        Ok(manifest) => manifest,
        Err(err) => {
            Error::FailedToReadCargo(err.to_string()).print_and_die();
        }
    };

    cargo_toml.dependencies.get("odra").unwrap().clone()
}
