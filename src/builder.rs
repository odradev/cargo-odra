use crate::backend::Backend;
use crate::command::parse_command_result;
use crate::odra_toml::OdraConf;
use crate::BuildCommand;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process::Command;

pub struct Builder {
    pub backend: Backend,
}

impl Builder {
    pub fn new(build: BuildCommand) -> Builder {
        let backend = Backend::load(build.backend);
        Builder { backend: backend }
    }

    pub fn builder_path(&self) -> String {
        format!(".builder_{}/", self.backend.name())
    }

    pub fn test_env_path(&self) -> String {
        format!("{}test_env", self.builder_path())
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
        let mut file = File::create(format!("{}/src/main.rs", self.builder_path())).unwrap();
        file.write_all(Builder::main_rs().as_bytes()).unwrap();
        self.create_build_files();
    }

    fn create_build_files(&self) {
        let conf = OdraConf::load();
        for (_, contract) in conf.contracts.into_iter() {
            let path = self.builder_path() + contract.path.as_str();
            if !Path::new(&path).exists() {
                let contents = Builder::def_rs()
                    .replace("#contract_fqn", &contract.fqn)
                    .replace("#contract_name", &contract.name)
                    .replace("#backend_name", &self.backend.name);
                let mut file = File::create(path).unwrap();
                file.write_all(contents.as_bytes()).unwrap();
            }
        }
    }

    fn def_rs() -> &'static str {
        r##"
fn main() {
    let contract_def = <#contract_fqn as odra::contract_def::HasContractDef>::contract_def();
    let code = odra_#backend_name_backend::codegen::gen_contract(contract_def, "#contract_fqn".to_string());

    use std::fs::File;
    use std::io::prelude::*;
    let mut file = File::create("src/#contract_name_wasm.rs").unwrap();
    file.write_all(&code.to_string().into_bytes()).unwrap();
}
        "##
    }

    fn main_rs() -> &'static str {
        r##"
fn main() {}
        "##
    }

    pub(crate) fn build_wasm(&self) {
        let conf = OdraConf::load();
        println!("Building wasm files...");
        for (_, contract) in conf.contracts.clone().into_iter() {
            // cargo run -p casper_builder --bin contract_def
            let command = Command::new("cargo")
                .current_dir(self.builder_path())
                .args([
                    "run",
                    "--bin",
                    format!("{}_build", &contract.name).as_str(),
                    "--no-default-features",
                    "--features",
                    "codegen",
                ])
                .status()
                .unwrap();

            parse_command_result(command, "Couldn't run wasm builder.")
        }

        for (_, contract) in conf.contracts.into_iter() {
            // cargo build --release --target wasm32-unknown-unknown -p casper_builder --bin plascoin
            let command = Command::new("cargo")
                .current_dir(self.builder_path())
                .args([
                    "build",
                    "--target",
                    "wasm32-unknown-unknown",
                    "--bin",
                    &contract.name,
                    "--release",
                    "--no-default-features",
                    "--features",
                    "wasm",
                ])
                .status()
                .unwrap();

            parse_command_result(
                command,
                format!("Couldn't build {} contract.", contract.name).as_str(),
            );
        }
    }

    pub(crate) fn copy_wasm_files(&self) {
        let conf = OdraConf::load();
        fs::create_dir_all("target/debug").unwrap();
        fs::create_dir_all("wasm").unwrap();
        for (_, contract) in conf.contracts.into_iter() {
            let source = format!(
                "{}target/wasm32-unknown-unknown/release/{}.wasm",
                self.builder_path(),
                contract.name
            );
            let target = format!("wasm/{}.wasm", contract.name);

            println!("Saving {}", target);

            Command::new("cp").args([source, target]).status().unwrap();

            let command = Command::new("wasm-strip")
                .current_dir("wasm")
                .arg(format!("{}.wasm", contract.name))
                .status()
                .expect("Couldn't run wasmstrip");

            match command.success() {
                true => {}
                false => {
                    println!("There was an error while running wasmstrip - Continuing anyway...");
                }
            }
        }
    }

    pub fn build_lib(&self) {
        let command = Command::new("cargo")
            .current_dir(self.builder_path())
            .args(["run", "--bin", "builder", "--release"])
            .status()
            .unwrap();

        parse_command_result(command, "Couldn't lib builder.");

        let source = format!(
            "{}target/release/deps/libodra_test_env.so",
            self.builder_path()
        );
        let target = "target/release/libodra_test_env.so".to_string();
        let target2 = "target/debug/libodra_test_env.so".to_string();

        println!("Saving {}", target);

        Command::new("cp")
            .args([source.clone(), target])
            .status()
            .unwrap();
        Command::new("cp").args([source, target2]).status().unwrap();
    }

    fn cargo_build() {
        println!("Running cargo build...");
        let command = Command::new("cargo").args(vec!["build"]).status().unwrap();
        parse_command_result(command, "Couldn't finish cargo build.")
    }

    pub(crate) fn build(&self) {
        self.backend.build_backend();
        self.prepare_builder(self.backend.name());
        crate::cargo_toml::builder_cargo_toml(self, &self.backend);
        self.build_wasm();
        self.copy_wasm_files();
        self.build_lib();
    }
}
