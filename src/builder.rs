use crate::backend::Backend;
use crate::{cargo_toml, odra_toml, BuildCommand};
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process::Command;

pub struct Builder {
    pub backend: Option<Backend>,
}

impl Builder {
    pub fn new(build: BuildCommand) -> Builder {
        match build.backend {
            None => Builder { backend: None },
            Some(backend_name) => {
                let backend = Backend::new(backend_name, build.repo_uri);
                Builder {
                    backend: Some(backend),
                }
            }
        }
    }

    pub fn builder_path(&self) -> String {
        match &self.backend {
            None => ".builder/".to_string(),
            Some(backend) => {
                format!(".builder_{}/", backend.name())
            }
        }
    }

    fn prepare_builder(&self, name: &String) {
        println!(
            "Preparing {} builder in {} directory...",
            name,
            self.builder_path()
        );

        if !Path::new(&self.builder_path()).is_dir() {
            fs::create_dir(self.builder_path()).unwrap();
        }
        let src_path = self.builder_path() + "/src";
        if !Path::new(&src_path).is_dir() {
            fs::create_dir(src_path).unwrap();
        }

        self.create_build_files(name);
    }

    fn create_build_files(&self, backend: &str) {
        let conf = odra_toml::load_odra_conf();
        for (_, contract) in conf.contracts.into_iter() {
            let path = self.builder_path() + contract.path.as_str();
            if !Path::new(&path).exists() {
                let contents = Builder::def_rs()
                    .replace("#contract_fqn", &contract.fqn)
                    .replace("#contract_name", &contract.name)
                    .replace("#backend_name", backend);
                let mut file = File::create(path).unwrap();
                file.write_all(contents.as_bytes()).unwrap();
            }
        }
    }

    fn def_rs() -> &'static str {
        r##"
fn main() {
    let contract_def = <#contract_fqn as odra::contract_def::HasContractDef>::contract_def();
    let code = #backend_name_backend::codegen::gen_contract(contract_def, "#contract_fqn".to_string());

    use std::fs::File;
    use std::io::prelude::*;
    let mut file = File::create("src/#contract_name_wasm.rs").unwrap();
    file.write_all(&code.to_string().into_bytes()).unwrap();
}
        "##
    }

    pub(crate) fn build_wasm(&self) {
        let conf = odra_toml::load_odra_conf();
        println!("Building wasm files...");
        for (_, contract) in conf.contracts.clone().into_iter() {
            // cargo run -p casper_builder --bin contract_def
            Command::new("cargo")
                .current_dir(self.builder_path())
                .args(["run", "--bin", format!("{}_build", &contract.name).as_str()])
                .status()
                .unwrap();
        }

        for (_, contract) in conf.contracts.into_iter() {
            // cargo build --release --target wasm32-unknown-unknown -p casper_builder --bin plascoin
            Command::new("cargo")
                .current_dir(self.builder_path())
                .args([
                    "build",
                    "--release",
                    "--target",
                    "wasm32-unknown-unknown",
                    "--bin",
                    &contract.name,
                ])
                .status()
                .unwrap();
        }
    }

    pub(crate) fn copy_wasm_files(&self, _name: &str) {
        let conf = odra_toml::load_odra_conf();
        fs::create_dir_all("target/debug").unwrap();
        fs::create_dir_all("wasm").unwrap();
        for (_, contract) in conf.contracts.into_iter() {
            println!("Copying wasm files...");
            Command::new("cp")
                .args([
                    format!(
                        "{}target/wasm32-unknown-unknown/release/{}.wasm",
                        self.builder_path(),
                        contract.name
                    )
                    .as_str(),
                    "wasm",
                ])
                .status()
                .unwrap();

            let wasm_output = Command::new("wasm-strip")
                .current_dir("wasm")
                .arg(format!("{}.wasm", contract.name))
                .output();

            match wasm_output {
                Ok(_) => {}
                Err(output) => {
                    println!(
                        "There was an error while running wasmstrip:\n{}\nContinuing anyway...",
                        output
                    );
                }
            }
        }
    }

    fn cargo_build() {
        println!("Running cargo build...");
        Command::new("cargo").args(vec!["build"]).output().unwrap();
    }

    pub(crate) fn build(&self) {
        match &self.backend {
            None => Builder::cargo_build(),
            Some(backend) => {
                backend.pull_backend();
                backend.build_backend();
                self.prepare_builder(backend.name());
                cargo_toml::build_cargo_toml(self, backend);
                self.build_wasm();
                self.copy_wasm_files(backend.name());
            }
        }
    }
}
