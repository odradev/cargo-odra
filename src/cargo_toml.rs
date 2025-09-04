//! Module containing functions used by Builder for managing its Cargo.toml file
use std::path::PathBuf;

use cargo_toml::{Dependency, DependencyDetail, Manifest};

use crate::{errors::Error, project::OdraLocation, utils::odra_latest_version};

/// Returns Cargo.toml as Manifest struct.
pub fn load_cargo_toml(path: &PathBuf) -> Manifest {
    match Manifest::from_path(path) {
        Ok(manifest) => manifest,
        Err(err) => {
            Error::FailedToReadCargo(err.to_string()).print_and_die();
        }
    }
}

pub fn odra_project_dependency(
    odra_location: &OdraLocation,
    crate_path: &str,
    init: bool,
) -> Dependency {
    let (version, path, git, branch) = match odra_location {
        OdraLocation::Local(path) => {
            let path = match init {
                true => path.clone(),
                false => PathBuf::from("..").join(path),
            };
            let path = path
                .join(crate_path)
                .into_os_string()
                .to_str()
                .unwrap()
                .to_string();
            (None, Some(path), None, None)
        }
        OdraLocation::Remote(repo, branch) => match branch {
            None => (Some(odra_latest_version()), None, None, None),
            Some(branch) => (None, None, Some(repo), Some(branch)),
        },
        OdraLocation::CratesIO(version) => (Some(version.clone()), None, None, None),
    };

    Dependency::Detailed(DependencyDetail {
        version,
        registry: None,
        registry_index: None,
        path,
        inherited: false,
        git: git.cloned(),
        branch: branch.cloned(),
        tag: None,
        rev: None,
        features: vec![],
        optional: false,
        default_features: false,
        package: None,
    })
}

pub fn odra_project_dependency_string(
    odra_location: &OdraLocation,
    crate_path: &str,
    init: bool,
) -> String {
    toml::to_string(&odra_project_dependency(odra_location, crate_path, init))
        .expect("Failed to serialize odra dependency.")
        .trim_end()
        .replace('\n', ", ")
}
