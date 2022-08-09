use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process::exit;

use cargo_toml::{Dependency, DependencyDetail, DepsSet};
use prettycli::{error, info};
use regex::Regex;
use serde_derive::{Deserialize, Serialize};

use crate::cargo_toml::builder_cargo_toml;
use crate::command::{cp, fmt as fmt_command, mkdir, wasm_strip};
use crate::odra_dependency::odra_details;
use crate::odra_toml::OdraConf;
use crate::{command, consts, AddBackendCommand, RemoveBackendCommand};

pub enum DependencyType {
    Local,
    Remote,
    Crates,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Backend {
    name: String,
    dependency_name: String,
    dependency: Dependency,
}

impl Backend {
    /// Main function that runs the whole workflow for backend
    pub fn build(&self) {
        self.prepare_builder(self.name());
        builder_cargo_toml(self);
        self.build_wasm();
        self.fmt();
        self.copy_wasm_files();
        self.build_lib();
    }

    /// Removes backend from Odra.toml
    pub fn remove(remove: RemoveBackendCommand) -> bool {
        let mut conf = OdraConf::load();
        if conf.backends.is_some() {
            let mut backends = conf.backends.unwrap();
            if backends.remove(&remove.name).is_some() {
                conf.backends = Some(backends);
                conf.save();
                return true;
            }
        }

        false
    }

    pub fn dependency_type(&self) -> DependencyType {
        match self.backend_dependency() {
            Dependency::Simple(_) => DependencyType::Crates,
            Dependency::Detailed(dependency_detail) => {
                if dependency_detail.path.is_some() {
                    return DependencyType::Local;
                }

                if dependency_detail.git.is_some() {
                    return DependencyType::Remote;
                }

                error("Unsupported dependency type for backend");
                exit(1);
            }
        }
    }

    pub fn builder_dependencies(&self) -> DepsSet {
        let mut dependencies = DepsSet::new();
        dependencies.insert(
            consts::ODRA_CRATE_NAME.to_string(),
            Backend::odra_dependency(),
        );
        dependencies.insert(OdraConf::load().name, Backend::project_dependency());
        dependencies.insert(
            format!("odra-{}-backend", self.dependency_name()),
            self.dependency("backend"),
        );
        dependencies.insert(
            format!("odra-{}-test-env", self.dependency_name()),
            self.dependency("test_env"),
        );
        dependencies
    }

    fn dependency(&self, folder: &str) -> Dependency {
        match self.dependency_type() {
            DependencyType::Local => {
                let mut dependency_detail = self.backend_dependency().detail().unwrap().clone();
                dependency_detail.path =
                    Some(format!("../{}{}", dependency_detail.path.unwrap(), folder));
                dependency_detail.optional = true;
                Dependency::Detailed(dependency_detail)
            }
            DependencyType::Remote => {
                let mut dependency_detail = self.backend_dependency().detail().unwrap().clone();
                dependency_detail.optional = true;
                Dependency::Detailed(dependency_detail)
            }
            DependencyType::Crates => {
                todo!()
            }
        }
    }

    fn odra_dependency() -> Dependency {
        let mut odra_details = odra_details().unwrap();
        odra_details.features = vec!["wasm".to_string()];
        odra_details.optional = true;
        odra_details.default_features = Some(false);
        if odra_details.path.is_some() {
            odra_details.path = Some(format!("../{}", odra_details.path.unwrap()));
        }
        Dependency::Detailed(odra_details)
    }

    fn project_dependency() -> Dependency {
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
            optional: true,
            default_features: Some(false),
            package: None,
        })
    }

    pub fn load(name: String) -> Backend {
        let conf = OdraConf::load();
        if conf.backends.is_none() {
            error("No backends configured.");
            exit(1);
        } else {
            let backends = conf.backends.unwrap();
            let backend = backends.get(&name);
            match backend {
                None => {
                    error("No such backend.");
                    exit(1);
                }
                Some(backend) => backend.clone(),
            }
        }
    }

    fn from_add_command(add: &AddBackendCommand) -> Backend {
        let dependency;
        if add.path.is_some() {
            dependency = Dependency::Detailed(DependencyDetail {
                version: None,
                registry: None,
                registry_index: None,
                path: add.path(),
                git: None,
                branch: None,
                tag: None,
                rev: None,
                features: vec![],
                optional: false,
                default_features: None,
                package: None,
            });
        } else if add.repo_uri.is_some() {
            dependency = Dependency::Detailed(DependencyDetail {
                version: None,
                registry: None,
                registry_index: None,
                path: None,
                git: add.repo_uri.clone(),
                branch: add.branch.clone(),
                tag: None,
                rev: None,
                features: vec![],
                optional: false,
                default_features: None,
                package: None,
            });
        } else {
            dependency = Dependency::Simple(add.package.clone());
        }

        Backend {
            name: add.name.clone(),
            dependency_name: add.package.clone(),
            dependency,
        }
    }

    pub fn add(add: AddBackendCommand) -> bool {
        let mut conf = OdraConf::load();
        let mut backends: HashMap<String, Backend>;
        if conf.backends.is_none() {
            backends = HashMap::new();
        } else {
            backends = conf.backends.unwrap();
        }
        if !backends.contains_key(&add.name) {
            backends.insert(add.name.clone(), Backend::from_add_command(&add));
            conf.backends = Some(backends);
            conf.save();
            true
        } else {
            false
        }
    }

    fn prepare_builder(&self, name: &String) {
        info(&format!(
            "Preparing {} builder in {} directory...",
            name,
            self.builder_path()
        ));

        if !Path::new(&self.builder_path()).is_dir() {
            fs::create_dir(self.builder_path()).unwrap();
        }
        let src_path = self.builder_path() + "/src";
        if !Path::new(&src_path).is_dir() {
            fs::create_dir(src_path).unwrap();
        }
        let mut file = File::create(format!("{}/src/main.rs", self.builder_path())).unwrap();
        file.write_all(Backend::main_rs().as_bytes()).unwrap();
        self.create_build_files();
    }

    fn create_build_files(&self) {
        let conf = OdraConf::load();
        for (_, contract) in conf.contracts.into_iter() {
            let path = self.builder_path() + contract.path.as_str();
            if !Path::new(&path).exists() {
                let contents = Backend::def_rs()
                    .replace("#contract_fqn", &contract.fqn)
                    .replace("#contract_name", &contract.name)
                    .replace("#backend_name", self.dependency_name());
                let mut file = File::create(path).unwrap();
                file.write_all(contents.as_bytes()).unwrap();
            }
        }
    }

    fn def_rs() -> &'static str {
        // TODO Use quote probably?
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

    fn build_wasm(&self) {
        let conf = OdraConf::load();
        info("Generating _wasm.rs files...");
        for (_, contract) in conf.contracts.clone().into_iter() {
            command::cargo(
                self.builder_path(),
                vec![
                    "run",
                    "--bin",
                    format!("{}_build", &contract.name).as_str(),
                    "--no-default-features",
                    "--features",
                    "codegen",
                ],
            );
        }

        info("Generating wasm files...");
        for (_, contract) in conf.contracts.into_iter() {
            // TODO Move to command.rs
            command::cargo(
                self.builder_path(),
                vec![
                    "build",
                    "--target",
                    "wasm32-unknown-unknown",
                    "--bin",
                    &contract.name,
                    "--release",
                    "--no-default-features",
                    "--features",
                    "wasm",
                ],
            );
        }
    }

    fn copy_wasm_files(&self) {
        info("Copying wasm files...");
        let conf = OdraConf::load();
        mkdir("target/debug");
        mkdir("wasm");
        for (_, contract) in conf.contracts.into_iter() {
            let source = format!(
                "{}target/wasm32-unknown-unknown/release/{}.wasm",
                self.builder_path(),
                contract.name
            );
            let target = format!("wasm/{}.wasm", contract.name);

            info(&format!("Saving {}", target));

            cp(&source, &target);
            wasm_strip(&contract.name);
        }
    }

    fn fmt(&self) {
        fmt_command(&self.builder_path());
    }

    fn build_lib(&self) {
        info("Building backend library...");
        command::cargo(
            self.builder_path(),
            vec![
                "run",
                "--bin",
                "builder",
                "--release",
                "--no-default-features",
                "--features=build",
            ],
        );

        let files = fs::read_dir(format!("{}target/release/deps/", self.builder_path())).unwrap();

        let pattern = format!(
            r"{}odra_test_env-.*\.{}",
            consts::PLATFORM_FILE_PREFIX,
            consts::PLATFORM_FILE_EXTENSION
        );
        let expression = Regex::new(&pattern).unwrap();

        let lib_filename = format!(
            r"{}odra_test_env.{}",
            consts::PLATFORM_FILE_PREFIX,
            consts::PLATFORM_FILE_EXTENSION
        );

        let mut source = lib_filename.clone();
        for entry in files {
            let filename = entry.unwrap().file_name();
            let filename = filename.to_str().unwrap().to_string();
            if expression.is_match(&filename) {
                source = filename.to_string();
            }
        }

        let source = format!("{}target/release/deps/{}", self.builder_path(), source);

        mkdir("target/release");
        mkdir("target/debug");
        cp(&source, &format!("target/release/{}", lib_filename));
        cp(&source, &format!("target/debug/{}", lib_filename));
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn dependency_name(&self) -> &String {
        &self.dependency_name
    }

    pub fn backend_dependency(&self) -> &Dependency {
        &self.dependency
    }

    pub fn builder_path(&self) -> String {
        format!(".builder_{}/", self.name())
    }
}
