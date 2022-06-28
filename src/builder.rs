use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process::Command;
use crate::odra_toml::OdraConf;

pub(crate) fn prepare_builder(backend: &String, conf: &OdraConf) {
    let builder_path = ".builder";
    if !Path::new(builder_path).is_dir() {
        fs::create_dir(builder_path).unwrap();
    }
    let src_path = builder_path.to_string() + "/src";
    if !Path::new(&src_path).is_dir() {
        fs::create_dir(src_path).unwrap();
    }

    create_build_files(backend, builder_path, conf);
}

fn create_build_files(backend: &String, builder_path: &str, conf: &OdraConf) {
    for (_, contract) in conf.contracts.clone().into_iter() {
        let path = builder_path.to_string() + "/" + contract.path.as_str();
        let contents = def_rs()
            .replace("#contract_fqn", &contract.fqn)
            .replace("#contract_name", &contract.name)
            .replace("#backend_name", backend);
        let mut file = File::create(path).unwrap();
        file.write_all(contents.as_bytes()).unwrap();
    }
}

fn def_rs() -> &'static str {
    "fn main() {
            let contract_def = <#contract_fqn as odra::contract_def::HasContractDef>::contract_def();
            let code = #backend_name_backend::codegen::gen_contract(contract_def);

            use std::fs::File;
            use std::io::prelude::*;
            let mut file = File::create(\"src/#contract_name_wasm.rs\").unwrap();
            file.write_all(&code.to_string().into_bytes()).unwrap();
        }"
}

pub(crate) fn build_wasm(conf: &OdraConf) {
    for (_, contract) in conf.contracts.clone().into_iter() {
        // cargo run -p casper_builder --bin contract_def
        Command::new("cargo")
            .current_dir(".builder")
            .args(["run", "--bin", format!("{}_build", &contract.name).as_str()])
            .status().unwrap();
    }

    // Fix gdyż odra robi sample_contract
    for (_, contract) in conf.contracts.clone().into_iter() {
        Command::new("sed")
            .current_dir(".builder/src")
            .args(["-i", "--", format!("s/sample_contract/{}/g", conf.name).as_str(), format!("{}_wasm.rs", &contract.name).as_str()])
            .status().unwrap();
    }

    for (_, contract) in conf.contracts.clone().into_iter() {
        // cargo build --release --target wasm32-unknown-unknown -p casper_builder --bin plascoin
        Command::new("cargo")
            .current_dir(".builder")
            .args(["build", "--release", "--target", "wasm32-unknown-unknown", "--bin", &contract.name])
            .status().unwrap();
    }
}

pub(crate) fn copy_wasm_files(conf: &OdraConf) {
    fs::create_dir_all("target/debug").unwrap();
    fs::create_dir_all("wasm").unwrap();
    if !Path::new("wasm/getter_proxy.wasm").exists() {
        let getter_proxy = attohttpc::get("https://github.com/odradev/cargo-odra/blob/master/getter_proxy.wasm?raw=true").send().unwrap().bytes().unwrap();
        fs::write("wasm/getter_proxy.wasm", getter_proxy).unwrap();
    }
    for (_, contract) in conf.contracts.clone().into_iter() {
        Command::new("cp")
        .args([format!(".builder/target/wasm32-unknown-unknown/release/{}.wasm", contract.name).as_str(), "wasm"])
        .status().unwrap();
    }
}