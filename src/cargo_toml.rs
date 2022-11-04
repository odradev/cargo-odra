//! Module containing functions used by Builder for managing its Cargo.toml file

use crate::errors::Error;
use crate::odra_toml::OdraToml;

use cargo_toml::{Dependency, DepsSet, FeatureSet, Manifest, Package, Product};
use std::fs::File;
use std::io::Write;
use std::path::Path;

/// Builds and saves Cargo.toml file for backend
pub fn builder_cargo_toml(builder_path: String, builder_deps: DepsSet, odra_toml: &OdraToml) {
    // TODO: Shorten and defaults.
    let mut bins = vec![];
    for (_, contract) in odra_toml.contracts.iter() {
        bins.push(Product {
            path: Some(contract.path.clone()),
            name: Some(format!("{}_build", contract.name.clone())),
            test: false,
            doctest: false,
            bench: false,
            doc: false,
            plugin: false,
            proc_macro: false,
            harness: false,
            edition: None,
            crate_type: None,
            required_features: vec![],
        });

        bins.push(Product {
            path: Some(contract.path.replace(".rs", "_wasm.rs")),
            name: Some(contract.name.clone()),
            test: false,
            doctest: false,
            bench: false,
            doc: false,
            plugin: false,
            proc_macro: false,
            harness: false,
            edition: None,
            crate_type: None,
            required_features: vec![],
        });
    }

    // TODO: Defaults
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

    let builder_cargo_toml_path = Path::new(&builder_path).join("Cargo.toml");
    let toml_contente = toml::to_string_pretty(&cargo_toml).unwrap();
    let mut cargo_toml_file = File::create(builder_cargo_toml_path).unwrap();
    cargo_toml_file.write_all(toml_contente.as_bytes()).unwrap();
}

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
