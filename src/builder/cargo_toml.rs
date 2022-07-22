/// File containing functions used by Builder for managing its Cargo.toml file
use crate::backend::Backend;
use crate::odra_dependency::odra_details;
use crate::odra_toml::OdraConf;
use crate::Builder;
use cargo_toml::{Dependency, DependencyDetail, DepsSet, Manifest, Package, Product};
use std::fmt::format;
use std::fs::File;
use std::io::{Read, Write};

pub(crate) fn build_cargo_toml(builder: &Builder, backend: &Backend) {
    // if Path::new(&(builder.builder_path() + "Cargo.toml")).exists() {
    //     return;
    // };
    let conf = OdraConf::load();

    let mut dependencies = DepsSet::new();
    let mut backend_dependency = backend.dependency.clone();
    match backend_dependency {
        Dependency::Simple(_) => {}
        Dependency::Detailed(mut dependency_detail) => {
            if dependency_detail.path.is_some() {
                dependency_detail = DependencyDetail {
                    version: dependency_detail.version.clone(),
                    registry: dependency_detail.registry.clone(),
                    registry_index: dependency_detail.registry_index.clone(),
                    path: Some(format!(
                        "../{}/backend",
                        dependency_detail.path.clone().unwrap()
                    )),
                    git: dependency_detail.git.clone(),
                    branch: dependency_detail.branch.clone(),
                    tag: dependency_detail.tag.clone(),
                    rev: dependency_detail.rev.clone(),
                    features: vec![],
                    optional: dependency_detail.optional.clone(),
                    default_features: Some(false),
                    package: dependency_detail.package.clone(),
                };
            }

            backend_dependency = Dependency::Detailed(dependency_detail);
        }
    }

    dependencies.insert(
        format!("odra-{}-backend", backend.dependency_name),
        backend_dependency,
    );
    // TODO: odra details should return Dependency
    let mut odra_dependency = odra_details().unwrap();
    odra_dependency.features = vec!["wasm".to_string()];
    odra_dependency.default_features = None;
    dependencies.insert("odra".to_string(), Dependency::Detailed(odra_dependency));

    dependencies.insert(
        conf.name,
        Dependency::Detailed(DependencyDetail {
            version: None,
            registry: None,
            registry_index: None,
            path: Some("..".to_string()),
            git: None,
            branch: None,
            tag: None,
            rev: None,
            features: vec!["wasm".to_string()],
            optional: false,
            default_features: None,
            package: None,
        }),
    );

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

    let cargo_toml: Manifest = cargo_toml::Manifest {
        // TODO: match version to cargo odra version.
        package: Some(Package::new("builder".to_string(), "1.0.0".to_string())),
        workspace: None,
        dependencies: dependencies,
        dev_dependencies: Default::default(),
        build_dependencies: Default::default(),
        target: Default::default(),
        features: Default::default(),
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
    let mut file = File::create(builder.builder_path() + "Cargo.toml").unwrap();
    file.write_all(toml.as_bytes()).unwrap();
}

fn cargo_toml() -> &'static str {
    r##"
[package]
name = "builder"
version = "0.1.0"
edition = "2021"

[dependencies]
#dependencies
odra = { git = "https://github.com/odradev/odra", default-features = false, features = ["wasm"] }
#package_name = { path = "..", default-features = false, features = ["wasm"] }

[build-dependencies]
quote = "1.0.18"
    "##
}

fn bin() -> &'static str {
    r##"
[[bin]]
name = "#contract_name_build"
path = "src/#contract_name.rs"

[[bin]]
name = "#contract_name"
path = "src/#contract_name_wasm.rs"
    "##
}
