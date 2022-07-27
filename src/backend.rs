use crate::command::parse_command_result;
use crate::odra_dependency::odra_details;
use crate::odra_toml::OdraConf;
use crate::{consts, AddBackendCommand};
use cargo_toml::{Dependency, DependencyDetail, DepsSet};
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::{exit, Command};
use std::{fs, process};

pub enum DependencyType {
    Local,
    Remote,
    Crates,
}

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

    pub fn dependency_type(&self) -> DependencyType {
        match &self.dependency {
            Dependency::Simple(_) => DependencyType::Crates,
            Dependency::Detailed(dependency_detail) => {
                if dependency_detail.path.is_some() {
                    return DependencyType::Local;
                }

                if dependency_detail.git.is_some() {
                    return DependencyType::Remote;
                }

                println!("Unsupported dependency for backend");
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
            format!("odra-{}-backend", self.dependency_name),
            self.backend_dependency(),
        );
        dependencies.insert(
            format!("odra-{}-test-env", self.dependency_name),
            self.test_env_dependency(),
        );
        dependencies
    }

    fn backend_dependency(&self) -> Dependency {
        let mut dependency_detail = self.dependency.detail().unwrap().clone();
        dependency_detail.path = Some(format!("../{}backend", dependency_detail.path.unwrap()));
        dependency_detail.optional = true;
        Dependency::Detailed(dependency_detail)
    }

    pub fn test_env_dependency(&self) -> Dependency {
        let mut dependency_detail = self.dependency.detail().unwrap().clone();
        dependency_detail.path = Some(format!("../{}test_env", dependency_detail.path.unwrap()));
        dependency_detail.optional = true;
        Dependency::Detailed(dependency_detail)
    }

    fn odra_dependency() -> Dependency {
        let mut odra_details = odra_details().unwrap();
        odra_details.features = vec!["wasm".to_string()];
        odra_details.optional = true;
        odra_details.default_features = None;
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
            optional: false,
            default_features: None,
            package: None,
        })
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
                path: add.path.clone(),
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
                git: None,
                branch: None,
                tag: None,
                rev: None,
                features: vec![],
                optional: false,
                default_features: None,
                package: None,
            });
        } else {
            dependency = Dependency::Simple("asdf".to_string());
        }

        Backend {
            name: add.name.clone(),
            dependency_name: add.name.clone(),
            dependency,
        }
    }

    pub fn new_local(name: String, path: String, branch: Option<String>) -> Backend {
        Backend {
            name: name.clone(),
            dependency_name: name.clone(),
            dependency: Dependency::Detailed(DependencyDetail {
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
                package: None,
            }),
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

    pub fn build_backend(&self) {}

    pub fn copy_libodra(&self) {
        println!("Copying lib...");
        // TODO: Make release test possible
        fs::create_dir_all("./target/debug").unwrap();

        let source = format!(
            "./builder_{}/test_env/target/debug/libodra_test_env.so",
            self.name
        );
        let target = "./target/debug/libodra_test_env.so";

        let command = Command::new("cp")
            .args(vec![source, target.to_string()])
            .status()
            .unwrap();

        parse_command_result(command, "Couldn't copy libodra.");
    }
}
