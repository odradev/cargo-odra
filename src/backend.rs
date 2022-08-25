use collections::hash_map::Entry::Vacant;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process::exit;
use std::{collections, fs};

use cargo_toml::{Dependency, DependencyDetail, DepsSet};
use comfy_table::Table;
use prettycli::{error, info};
use regex::Regex;
use serde_derive::{Deserialize, Serialize};

use crate::cargo_toml::builder_cargo_toml;
use crate::command::{cp, fmt as fmt_command, mkdir, wasm_strip};
use crate::odra_dependency::odra_dependency;
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
        self.check_requirements();
        self.prepare_builder(self.name());
        builder_cargo_toml(self);
        self.build_wasm();
        self.fmt();
        self.copy_wasm_files();
        self.build_lib();
    }

    /// Prints out a table containing all backends
    pub fn list() {
        let backends = OdraConf::load().backends.unwrap_or_else(|| {
            println!("No backends configured.");
            exit(1);
        });

        let mut table = Table::new();
        table.set_header(vec!["Name", "Package", "Dependency"]);
        for (_, backend) in backends {
            table.add_row(vec![
                backend.name,
                backend.dependency_name,
                toml::to_string(&backend.dependency).unwrap(),
            ]);
        }

        println!("{}", table);
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

    /// Adds backend to Odra.toml
    pub fn add(add: AddBackendCommand) -> bool {
        let mut conf = OdraConf::load();
        let mut backends: HashMap<String, Backend>;

        // If no name was passed, we use package
        let name = match &add.name {
            None => add.package.clone(),
            Some(name) => name.clone(),
        };

        if conf.backends.is_none() {
            backends = HashMap::new();
        } else {
            backends = conf.backends.unwrap();
        }

        if let Vacant(e) = backends.entry(name) {
            e.insert(Backend::from_add_command(&add));
            conf.backends = Some(backends);
            conf.save();
            true
        } else {
            false
        }
    }

    /// Returns the type of backend dependency
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

                DependencyType::Crates
            }
        }
    }

    /// Returns a set of dependencies used by backend
    pub fn builder_dependencies(&self) -> DepsSet {
        let mut dependencies = DepsSet::new();
        dependencies.insert(
            consts::ODRA_CRATE_NAME.to_string(),
            Backend::odra_dependency(),
        );
        dependencies.insert(OdraConf::load().name, Backend::project_dependency());
        dependencies.insert(
            format!("odra-{}-backend", self.package()),
            self.dependency("backend"),
        );
        dependencies.insert(
            format!("odra-{}-test-env", self.package()),
            self.dependency("test_env"),
        );
        dependencies
    }

    /// Loads a backend from Odra.toml file
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
                let mut dependency_detail = self.backend_dependency().detail().unwrap().clone();
                dependency_detail.optional = true;
                Dependency::Detailed(dependency_detail)
            }
        }
    }

    /// Returns Odra dependency tailored for use by backend (optional set to true and wasm feature
    /// enabled)
    fn odra_dependency() -> Dependency {
        let dependency = odra_dependency();
        match dependency {
            Dependency::Simple(simple) => Dependency::Detailed(DependencyDetail {
                version: Some(simple),
                optional: true,
                ..Default::default()
            }),
            Dependency::Detailed(mut odra_details) => {
                odra_details.features = vec!["wasm".to_string()];
                odra_details.optional = true;
                odra_details.default_features = Some(false);
                if odra_details.path.is_some() {
                    odra_details.path = Some(format!("../{}", odra_details.path.unwrap()));
                }
                Dependency::Detailed(odra_details)
            }
        }
    }

    fn project_dependency() -> Dependency {
        Dependency::Detailed(DependencyDetail {
            path: Some("..".to_string()),
            features: vec!["wasm".to_string()],
            optional: true,
            default_features: Some(false),
            ..Default::default()
        })
    }

    fn from_add_command(add: &AddBackendCommand) -> Backend {
        let dependency;
        if add.path.is_some() {
            dependency = Dependency::Detailed(DependencyDetail {
                path: add.path(),
                ..Default::default()
            });
        } else if add.repo_uri.is_some() {
            dependency = Dependency::Detailed(DependencyDetail {
                git: add.repo_uri.clone(),
                branch: add.branch.clone(),
                ..Default::default()
            });
        } else {
            let version = Some("*".to_string());
            dependency = Dependency::Detailed(DependencyDetail {
                version,
                ..Default::default()
            });
        }

        let name = match &add.name {
            None => add.package.clone(),
            Some(name) => name.clone(),
        };

        Backend {
            name,
            dependency_name: add.package.clone(),
            dependency,
        }
    }

    fn check_requirements(&self) {
        if !command::command_output("rustup target list --installed")
            .contains("wasm32-unknown-unknown")
        {
            error("wasm32-unknown-unknown target is not present, install it by executing:");
            println!("rustup target add wasm32-unknown-unknown");
            exit(1);
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
                    .replace("#backend_name", self.package());
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

    /// Returns the name of the backend. It is a name used internally by a project. Most of the time
    /// it will be the same as package. It is useful, when you want to add more than one of the same
    /// backends, for example in different versions, from github or one stored locally (e.g.
    /// `casper`, `casper-local`, `casper-develop`).
    pub fn name(&self) -> &String {
        &self.name
    }

    /// Returns name of the dependency. For example `casper` will be later used to generate
    /// `odra-casper-backend` as a name of dependency on crates.io
    pub fn package(&self) -> &String {
        &self.dependency_name
    }

    /// Returns backend's dependency
    pub fn backend_dependency(&self) -> &Dependency {
        &self.dependency
    }

    /// Returns a path where builder lives
    pub fn builder_path(&self) -> String {
        format!(".builder_{}/", self.name())
    }
}
