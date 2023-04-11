//! Module containing functions used by Builder for managing its Cargo.toml file

use std::path::PathBuf;
use cargo_toml::{Dependency, DepsSet, Edition, FeatureSet, Manifest, Package, Product, Workspace};

use crate::project::{Member, Project};
use crate::{
    command,
    errors::Error,
    odra_toml::OdraToml,
    paths::{self, BuilderPaths},
};

/// Builds and saves Cargo.toml file for backend.
pub fn builder_cargo_toml(
    builder_paths: &BuilderPaths,
    builder_deps: DepsSet,
    odra_toml: &OdraToml,
) {
    let default_bin = Product {
        test: false,
        doctest: false,
        bench: false,
        doc: false,
        edition: Some(Edition::E2021),
        ..Default::default()
    };

    let mut bins = vec![];
    for contract in odra_toml.contracts.iter() {
        let build_name = format!("{}_build", contract.name.clone());
        let path = builder_paths
            .relative()
            .wasm_build_as_string(&contract.name);
        bins.push(Product {
            path: Some(path),
            name: Some(build_name),
            ..default_bin.clone()
        });

        let path = builder_paths
            .relative()
            .wasm_source_as_string(&contract.name);
        bins.push(Product {
            path: Some(path),
            name: Some(contract.name.clone()),
            ..default_bin.clone()
        });
    }

    #[allow(deprecated)]
    let cargo_toml: Manifest = cargo_toml::Manifest {
        package: Some(Package::new("builder".to_string(), "1.0.0".to_string())),
        workspace: Some(Workspace {
            members: vec![],
            default_members: vec![],
            exclude: vec![],
            metadata: None,
            resolver: None,
        }),
        dependencies: builder_deps,
        dev_dependencies: Default::default(),
        build_dependencies: Default::default(),
        target: Default::default(),
        features: FeatureSet::new(),
        patch: Default::default(),
        lib: None,
        profile: Default::default(),
        badges: Default::default(),
        bin: bins,
        bench: vec![],
        test: vec![],
        example: vec![],
        replace: Default::default(),
    };

    let toml_content = toml::to_string_pretty(&cargo_toml).unwrap();
    command::write_to_file(builder_paths.cargo_toml(), &toml_content);
}

/// Returns Dependency of Odra, taken from project's Cargo.toml.
pub fn odra_dependency(cargo_toml_path: PathBuf) -> Dependency {
    load_cargo_toml(cargo_toml_path).dependencies.get("odra").unwrap().clone()
}

/// Returns project's name from Cargo.toml.
pub fn project_name() -> String {
    load_main_cargo_toml().package.unwrap().name
}

pub fn members() -> Vec<(String, String)> {
    match load_main_cargo_toml().workspace {
        Some(workspace) => workspace
            .members
            .iter()
            .map(|member| (member.clone(), member.clone()))
            .collect(),
        None => vec![(project_name(), "".to_string())],
    }
}

/// Returns Cargo.toml as Manifest struct.
pub fn load_main_cargo_toml() -> Manifest {
    load_cargo_toml(Project::find_cargo_toml().unwrap())
}

/// Returns Cargo.toml as Manifest struct.
pub fn load_cargo_toml(path: PathBuf) -> Manifest {
    match Manifest::from_path(path) {
        Ok(manifest) => manifest,
        Err(err) => {
            Error::FailedToReadCargo(err.to_string()).print_and_die();
        }
    }
}
