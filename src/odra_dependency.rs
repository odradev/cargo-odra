/// Functions used to read and manage Cargo.toml of a project using odra
use std::process;
use cargo_toml::{DependencyDetail, Manifest};

pub fn odra_details() -> Option<DependencyDetail> {
    let cargo_toml = match Manifest::from_path("Cargo.toml") {
        Ok(manifest) => { manifest }
        Err(_) => {
            println!("Cargo.toml not found, exiting.");
            process::exit(2);
        }
    };

    match cargo_toml.dependencies.get("odra").unwrap().detail() {
        None => { None }
        Some(dd) => { Some(dd.clone()) }
    }
}