use std::fs::File;
use std::io::Write;
use crate::odra_toml::OdraConf;

pub(crate) fn build_cargo_toml(backend: &String, conf: &OdraConf) {
    let mut cargo_toml = cargo_toml()
        .replace("#package_name", &conf.name)
        .replace("#backend_name", backend);

    for (_, contract) in conf.contracts.clone().into_iter() {
        cargo_toml += bin().replace("#contract_name", contract.name.as_str()).as_str();
    }

    let mut file = File::create(".builder/Cargo.toml").unwrap();
    file.write_all(cargo_toml.as_bytes()).unwrap();
}

fn cargo_toml() -> &'static str {
    r#"
[package]
name = "builder"
version = "0.1.0"
edition = "2021"

[dependencies]
#backend_name_backend = { git = "https://github.com/odradev/odra-casper", default-features = false, features = ["codegen", "backend"] }
odra = { git = "https://github.com/odradev/odra", default-features = false, features = ["wasm"] }
#package_name = { path = "..", default-features = false, features = ["wasm"] }

[build-dependencies]
quote = "1.0.18"
    "#
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