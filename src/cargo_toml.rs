//! Module containing functions used by Builder for managing its Cargo.toml file
use std::path::PathBuf;

use cargo_toml::{Dependency, DependencyDetail, Manifest};

use crate::{command, errors::Error, project::OdraLocation, utils::odra_latest_version};

/// Discovers the single binary ending with `_build_contract` in the given `Cargo.toml`.
/// Falls back to `default` when none is found.
/// Errors out when more than one match is found (ambiguous).
pub fn discover_build_contract_bin(cargo_toml_path: &PathBuf, default: &str) -> String {
    let mut manifest = load_cargo_toml(cargo_toml_path);
    let _ = manifest.complete_from_path(cargo_toml_path);
    let matches: Vec<String> = manifest
        .bin
        .iter()
        .filter_map(|b| b.name.clone())
        .filter(|name| name.ends_with("_build_contract"))
        .collect();
    match matches.as_slice() {
        [] => default.to_string(),
        [name] => name.clone(),
        _ => Error::MultipleBuildContractBins(
            cargo_toml_path.display().to_string(),
            matches.join(", "),
        )
        .print_and_die(),
    }
}

/// Returns Cargo.toml as Manifest struct.
pub fn load_cargo_toml(path: &PathBuf) -> Manifest {
    match Manifest::from_path(path) {
        Ok(manifest) => manifest,
        Err(err) => {
            Error::FailedToReadCargo(err.to_string()).print_and_die();
        }
    }
}

/// Saves configuration into Odra.toml file.
pub fn save_cargo_toml(path: PathBuf, cargo_toml: &Manifest) -> Result<(), Error> {
    let content = toml::to_string(cargo_toml)?;
    command::write_to_file(path, &content)
}

pub fn odra_project_dependency(
    odra_location: &OdraLocation,
    crate_path: &str,
    optional: bool,
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
        optional,
        default_features: false,
        package: None,
    })
}

pub fn odra_project_dependency_string(
    odra_location: &OdraLocation,
    crate_path: &str,
    init: bool,
) -> String {
    toml::to_string(&odra_project_dependency(
        odra_location,
        crate_path,
        false,
        init,
    ))
    .expect("Failed to serialize odra dependency.")
    .trim_end()
    .replace('\n', ", ")
}

pub fn opt_odra_project_dependency_string(
    odra_location: &OdraLocation,
    crate_path: &str,
    init: bool,
) -> String {
    toml::to_string(&odra_project_dependency(
        odra_location,
        crate_path,
        true,
        init,
    ))
    .expect("Failed to serialize odra dependency.")
    .trim_end()
    .replace('\n', ", ")
}
