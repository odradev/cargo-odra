mod builder;

use std::collections::HashMap;
use std::{fs, process};
use std::process::Command;
use cargo_toml::{Dependency, DependencyDetail};
use serde_derive::{Deserialize, Serialize};
use crate::AddBackendCommand;
use crate::odra_toml::{Contract, OdraConf};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Backend {
    pub name: String,
    pub dependency_name: String,
    pub dependency: Dependency,
}

impl Backend {
    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn path(&self) -> &String {
        todo!();
    }

    pub fn load(name: String) -> Backend {
        let conf = OdraConf::load();
        if conf.backends.is_none() {
            println!("No backends configured.");
            process::exit(1);
        } else {
            let backends = conf.backends.unwrap();
            let backend = backends.get(&name);
            match backend {
                None => {
                    println!("No such backend.");
                    process::exit(1);
                }
                Some(backend) => {
                    backend.clone()
                }
            }
        }
    }

    fn from_add_command(add: &AddBackendCommand) -> Backend {
        let dependency;
        if add.path.is_some() {
            dependency = Dependency::Detailed(
                DependencyDetail {
                    version: None,
                    registry: None,
                    registry_index: None,
                    path: add.path.clone(),
                    git: None,
                    branch: None,
                    tag: None,
                    rev: None,
                    features: vec![],
                    optional: false,
                    default_features: None,
                    package: None
                }
            );
        } else if add.repo_uri.is_some() {
            dependency = Dependency::Detailed(
                DependencyDetail {
                    version: None,
                    registry: None,
                    registry_index: None,
                    path: None,
                    git: None,
                    branch: None,
                    tag: None,
                    rev: None,
                    features: vec![],
                    optional: false,
                    default_features: None,
                    package: None
                }
            );
        } else {
            dependency = Dependency::Simple("asdf".to_string());
        }

        Backend {
            name: add.name.clone(),
            dependency_name: add.name.clone(),
            dependency
        }
    }

    pub fn new_local(name: String, path: String, branch: Option<String>) -> Backend {
        Backend {
            name: name.clone(),
            dependency_name: name.clone(),
            dependency: Dependency::Detailed(
                DependencyDetail {
                    version: None,
                    registry: None,
                    registry_index: None,
                    path: Some(path),
                    git: None,
                    branch,
                    tag: None,
                    rev: None,
                    features: vec![],
                    optional: false,
                    default_features: Some(false),
                    package: None
                }
            )
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

    pub fn build_backend(&self) {

    }

    pub fn save(&self) {

    }

    pub fn from_dependency(dependency: &Dependency) {

    }

    pub fn from_dependency_detail(dependency_detail: &DependencyDetail) {

    }

    fn test_env_path(&self) -> String {
        todo!();
    }

    pub fn copy_libodra(&self) {
        println!("Copying lib...");
        fs::create_dir_all("./target/debug").unwrap();

        let source = format!("{}target/debug/libodra_test_env.so", self.test_env_path());
        let target = "./target/debug/libodra_test_env.so";

        Command::new("cp")
            .args(vec![source, target.to_string()])
            .status()
            .expect("Couldn't copy lib");
    }
}
