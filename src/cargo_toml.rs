/// File containing functions used by Builder for managing its Cargo.toml file
use crate::backend::Backend;

use crate::odra_toml::OdraConf;
use cargo_toml::{FeatureSet, Manifest, Package, Product};

use std::fs::File;
use std::io::Write;

/// Builds and saves Cargo.toml file for backend
pub fn builder_cargo_toml(backend: &Backend) {
    let conf = OdraConf::load();

    let mut bins = vec![];
    for (_, contract) in conf.contracts.into_iter() {
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
            name: Some(contract.name),
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
    let project_name = OdraConf::load().name;

    let mut features = FeatureSet::new();
    features.insert("default".to_string(), vec!["build".to_string()]);
    features.insert(
        "build".to_string(),
        vec![format!("odra-{}-test-env", backend.package())],
    );
    features.insert(
        "codegen".to_string(),
        vec![
            format!("odra-{}-backend", backend.package()),
            project_name.clone(),
            "odra".to_string(),
        ],
    );
    features.insert(
        "wasm".to_string(),
        vec![
            "odra/wasm".to_string(),
            project_name,
            format!("odra-{}-backend", backend.package()),
        ],
    );

    let cargo_toml: Manifest = cargo_toml::Manifest {
        package: Some(Package::new("builder".to_string(), "1.0.0".to_string())),
        workspace: None,
        dependencies: backend.builder_dependencies(),
        dev_dependencies: Default::default(),
        build_dependencies: Default::default(),
        target: Default::default(),
        features,
        patch: Default::default(),
        lib: None,
        profile: Default::default(),
        badges: Default::default(),
        bin: bins,
        bench: vec![],
        test: vec![],
        example: vec![],
    };

    let toml = toml::to_string(&cargo_toml).unwrap();

    let mut file = File::create(backend.builder_path() + "Cargo.toml").unwrap();
    file.write_all(toml.as_bytes()).unwrap();
}
