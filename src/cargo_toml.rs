//! Module containing functions used by Builder for managing its Cargo.toml file
use std::path::PathBuf;

use cargo_toml::Manifest;

use crate::errors::Error;

/// Returns Cargo.toml as Manifest struct.
pub fn load_cargo_toml(path: &PathBuf) -> Manifest {
    match Manifest::from_path(path) {
        Ok(manifest) => manifest,
        Err(err) => {
            Error::FailedToReadCargo(err.to_string()).print_and_die();
        }
    }
}
