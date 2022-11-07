//! Module containing functions used by Builder for managing its Cargo.toml file

use crate::errors::Error;
use crate::odra_toml::OdraToml;
use crate::paths::BuilderPaths;

use cargo_toml::{Dependency, DepsSet, Edition, FeatureSet, Manifest, Package, Product};
use std::fs::File;
use std::io::Write;

/// Builds and saves Cargo.toml file for backend
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
        bins.push(Product {
            path: Some(
                builder_paths
                    .relative()
                    .wasm_build_as_string(&contract.name),
            ),
            name: Some(build_name),
            ..default_bin.clone()
        });

        bins.push(Product {
            path: Some(builder_paths.relative().wasm_source_file(&contract.name)),
            name: Some(contract.name.clone()),
            ..default_bin.clone()
        });
    }

    #[allow(deprecated)]
    let cargo_toml: Manifest = cargo_toml::Manifest {
        package: Some(Package::new("builder".to_string(), "1.0.0".to_string())),
        workspace: None,
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

    let toml_contente = toml::to_string_pretty(&cargo_toml).unwrap();
    let mut cargo_toml_file = File::create(builder_paths.cargo_toml()).unwrap();
    cargo_toml_file.write_all(toml_contente.as_bytes()).unwrap();
}

/// Returns Dependency of Odra, taken from project's Cargo.toml
pub fn odra_dependency() -> Dependency {
    load_cargo_toml().dependencies.get("odra").unwrap().clone()
}

pub fn project_name() -> String {
    load_cargo_toml().package.unwrap().name
}

fn load_cargo_toml() -> Manifest {
    match Manifest::from_path("Cargo.toml") {
        Ok(manifest) => manifest,
        Err(err) => {
            Error::FailedToReadCargo(err.to_string()).print_and_die();
        }
    }
}
