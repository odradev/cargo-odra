/// File containing functions used by Builder for managing its Cargo.toml file
use crate::backend::Backend;
use crate::odra_toml::load_odra_conf;
use crate::Builder;
use std::fs::File;
use std::io::Write;

pub(crate) fn build_cargo_toml(builder: &Builder, backend: &Backend) {
    // if Path::new(&(builder.builder_path() + "Cargo.toml")).exists() {
    //     return;
    // };
    let conf = load_odra_conf();
    let mut cargo_toml = cargo_toml()
        .replace("#package_name", &conf.name)
        .replace("#backend_name", backend.name())
        .replace("#backend_repo", backend.path());

    for (_, contract) in conf.contracts.into_iter() {
        cargo_toml += bin()
            .replace("#contract_name", contract.name.as_str())
            .as_str();
    }

    let mut file = File::create(builder.builder_path() + "Cargo.toml").unwrap();
    file.write_all(cargo_toml.as_bytes()).unwrap();
}

fn cargo_toml() -> &'static str {
    r##"
[package]
name = "builder"
version = "0.1.0"
edition = "2021"

[dependencies]
#backend_name_backend = { git = "#backend_repo", default-features = false, features = ["codegen", "backend"] }
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
