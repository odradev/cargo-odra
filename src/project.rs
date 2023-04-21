use std::{env, path::PathBuf};

use cargo_generate::{GenerateArgs, TemplatePath, Vcs};
use cargo_toml::{Dependency, DependencyDetail};
use chrono::Utc;
use ureq::serde_json;

use crate::{
    actions::{build::BuildAction, generate::GenerateAction, init::InitAction, test::TestAction},
    cargo_toml::load_cargo_toml,
    cli::{GenerateCommand, TestCommand},
    command::replace_in_file,
    consts::{ODRA_GITHUB_API_DATA, ODRA_TEMPLATE_GH_REPO},
    errors::Error,
    odra_toml::OdraToml,
    paths,
};

/// Struct representing the whole project.
#[derive(Debug, Clone)]
pub struct Project {
    /// Name of the project.
    pub name: String,
    /// Root directory of the project.
    pub project_root: PathBuf,
    /// Path to the main Cargo.toml file.
    pub cargo_toml_location: PathBuf,
    /// Path to the Odra.toml file.
    pub odra_toml_location: PathBuf,
    /// Members of the project.
    pub members: Vec<Member>,
}

impl Project {
    /// Create a new Project.
    pub fn init(init_action: InitAction) {
        if init_action.generate {
            Self::generate_project(init_action);
        }
    }

    /// Generates a new project.
    pub fn generate_project(init_action: InitAction) {
        if init_action.init {
            Self::assert_dir_is_empty(init_action.current_dir.clone());
        }

        let odra_location = Self::odra_location(init_action.source);

        let template_path = match odra_location.clone() {
            OdraLocation::Local(local_path) => TemplatePath {
                auto_path: Some(local_path.as_os_str().to_str().unwrap().to_string()),
                subfolder: Some(format!("templates/{}", init_action.template)),
                test: false,
                git: None,
                branch: None,
                tag: None,
                path: None,
                favorite: None,
            },
            OdraLocation::Remote(repo, branch) => TemplatePath {
                auto_path: Some(repo),
                subfolder: Some(format!("templates/{}", init_action.template)),
                test: false,
                git: None,
                branch,
                tag: None,
                path: None,
                favorite: None,
            },
        };

        cargo_generate::generate(GenerateArgs {
            template_path,
            list_favorites: false,
            name: Some(paths::to_snake_case(&init_action.project_name)),
            force: true,
            verbose: false,
            template_values_file: None,
            silent: false,
            config: None,
            vcs: Some(Vcs::Git),
            lib: false,
            bin: false,
            ssh_identity: None,
            define: vec![format!("date={}", Utc::now().format("%Y-%m-%d"))],
            init: init_action.init,
            destination: None,
            force_git_init: false,
            allow_commands: false,
            overwrite: false,
            other_args: None,
        })
        .unwrap_or_else(|e| {
            Error::FailedToGenerateProjectFromTemplate(e.to_string()).print_and_die();
        });

        let cargo_toml_path = match init_action.init {
            true => {
                let mut path = init_action.current_dir;
                path.push("Cargo.toml");
                path
            }
            false => {
                let mut path = init_action.current_dir;
                path.push(paths::to_snake_case(&init_action.project_name));
                path.push("Cargo.toml");
                path
            }
        };

        replace_in_file(
            cargo_toml_path,
            "#odra_dependency",
            format!(
                "odra = {{ {} }}",
                toml::to_string(&Self::odra_dependency(odra_location, init_action.init))
                    .unwrap()
                    .trim_end()
                    .replace('\n', ", ")
            )
            .as_str(),
        );
    }

    /// Detects an existing project.
    pub fn detect(path: PathBuf) -> Project {
        let odra_toml_path = Self::find_odra_toml(path.clone()).unwrap_or_else(|| {
            Error::NotAnOdraProject.print_and_die();
        });
        let cargo_toml_path = Self::find_cargo_toml(path).unwrap_or_else(|| {
            Error::NotAnOdraProject.print_and_die();
        });
        let root = odra_toml_path.parent().unwrap().to_path_buf();
        let members = Self::find_members(&cargo_toml_path);
        let name = match load_cargo_toml(&cargo_toml_path).package {
            None => {
                let cwd = env::current_dir().unwrap();
                cwd.strip_prefix(cwd.parent().unwrap())
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string()
            }
            Some(package) => package.name,
        };
        Project {
            name,
            project_root: root,
            cargo_toml_location: cargo_toml_path,
            odra_toml_location: odra_toml_path,
            members,
        }
    }

    /// Builds the project
    pub fn build(&self, backend: String, contract_name: Option<String>) {
        BuildAction::new(self, backend, contract_name).build();
    }

    /// Runs test in the Project.
    pub fn test(&self, test: TestCommand) {
        TestAction::new(self, test.backend, test.args, test.skip_build).test();
    }

    /// Generates a new contract in the Project.
    pub fn generate(&self, generate: GenerateCommand) {
        GenerateAction::new(self, generate.contract_name, generate.module).generate_contract();
    }

    /// Root directory of the module.
    /// If the project does not use workspaces, the root directory is the same as the project root.
    pub fn module_root(&self, module_name: Option<String>) -> PathBuf {
        match module_name {
            None => self.project_root.clone(),
            Some(module_name) => self
                .members
                .iter()
                .find(|member| member.name == module_name)
                .unwrap_or_else(|| {
                    Error::ModuleNotFound(module_name).print_and_die();
                })
                .root
                .clone(),
        }
    }

    /// Name of the module.
    /// If there is no module name, the project name is returned.
    pub fn module_name(&self, module_name: Option<String>) -> String {
        match module_name {
            None => self.name.clone(),
            Some(module_name) => self
                .members
                .iter()
                .find(|member| member.name == module_name)
                .unwrap_or_else(|| {
                    Error::ModuleNotFound(module_name).print_and_die();
                })
                .name
                .clone(),
        }
    }

    /// Searches for main Projects' Cargo.toml.
    pub fn find_cargo_toml(path: PathBuf) -> Option<PathBuf> {
        match Self::find_file_upwards("Odra.toml", path) {
            None => None,
            Some(odra_toml_path) => {
                let cargo_toml_path = Some(odra_toml_path.parent().unwrap().join("Cargo.toml"));
                if cargo_toml_path.as_ref().unwrap().exists() {
                    cargo_toml_path
                } else {
                    None
                }
            }
        }
    }

    /// Root directory of the Project.
    pub fn project_root(&self) -> PathBuf {
        self.project_root.clone()
    }

    /// Check if the project is a workspace.
    pub fn is_workspace(&self) -> bool {
        self.members.len() > 1 || self.members[0].root != self.project_root()
    }

    /// Returns project's OdraToml.
    pub fn odra_toml(&self) -> OdraToml {
        OdraToml::load(&self.odra_toml_location)
    }

    fn assert_dir_is_empty(dir: PathBuf) {
        if dir.read_dir().unwrap().next().is_some() {
            Error::CurrentDirIsNotEmpty.print_and_die();
        }
    }

    fn find_members(cargo_toml_path: &PathBuf) -> Vec<Member> {
        Self::detect_members(cargo_toml_path)
            .iter()
            .map(|member| {
                let root = cargo_toml_path.parent().unwrap().join(member.clone().1);
                let cargo_toml = root.join("Cargo.toml");
                Member {
                    name: member.clone().0,
                    root,
                    cargo_toml,
                }
            })
            .collect()
    }

    fn find_odra_toml(path: PathBuf) -> Option<PathBuf> {
        Self::find_file_upwards("Odra.toml", path)
    }

    fn find_file_upwards(filename: &str, path: PathBuf) -> Option<PathBuf> {
        let mut path = path;
        loop {
            let file_path = path.join(filename);
            if file_path.exists() {
                return Some(file_path);
            }
            if path.parent().is_none() || path == path.parent().unwrap() {
                return None;
            }
            path = path.parent().unwrap().to_path_buf();
        }
    }

    fn detect_members(cargo_toml_path: &PathBuf) -> Vec<(String, String)> {
        match load_cargo_toml(cargo_toml_path).workspace {
            Some(workspace) => workspace
                .members
                .iter()
                .map(|member| (member.clone(), member.clone()))
                .collect(),
            None => vec![(Self::detect_project_name(cargo_toml_path), "".to_string())],
        }
    }

    fn detect_project_name(cargo_toml_path: &PathBuf) -> String {
        load_cargo_toml(cargo_toml_path).package.unwrap().name
    }

    fn odra_dependency(odra_location: OdraLocation, init: bool) -> Dependency {
        let (version, path, git, branch) = match odra_location {
            OdraLocation::Local(path) => {
                let path = match init {
                    true => path,
                    false => PathBuf::from("..").join(path),
                };
                let path = path
                    .join("core")
                    .into_os_string()
                    .to_str()
                    .unwrap()
                    .to_string();
                (None, Some(path), None, None)
            }
            OdraLocation::Remote(repo, branch) => match branch {
                None => (Some(Self::odra_latest_version()), None, None, None),
                Some(branch) => (None, None, Some(repo), Some(branch)),
            },
        };

        Dependency::Detailed(DependencyDetail {
            version,
            registry: None,
            registry_index: None,
            path,
            inherited: false,
            git,
            branch,
            tag: None,
            rev: None,
            features: vec![],
            optional: false,
            default_features: false,
            package: None,
        })
    }

    fn odra_latest_version() -> String {
        let response: serde_json::Value = ureq::get(ODRA_GITHUB_API_DATA)
            .call()
            .unwrap_or_else(|_| {
                Error::FailedToFetchTemplate(ODRA_GITHUB_API_DATA.to_string()).print_and_die()
            })
            .into_json()
            .unwrap_or_else(|_| {
                Error::FailedToParseTemplate(ODRA_GITHUB_API_DATA.to_string()).print_and_die()
            });
        response["tag_name"].as_str().unwrap().to_string()
    }

    pub fn project_odra_location(&self) -> OdraLocation {
        let cargo_toml = load_cargo_toml(&self.cargo_toml_location);
        let odra_dependency = cargo_toml
            .dependencies
            .iter()
            .find(|dependency| dependency.0 == "odra")
            .unwrap()
            .1
            .clone();
        match odra_dependency {
            Dependency::Detailed(DependencyDetail {
                path: Some(path),
                git: None,
                ..
            }) => {
                let path = PathBuf::from(path);
                OdraLocation::Local(PathBuf::from(path.parent().unwrap()))
            }
            Dependency::Detailed(DependencyDetail {
                git: Some(git),
                branch: Some(branch),
                ..
            }) => OdraLocation::Remote(git, Some(branch)),
            Dependency::Detailed(DependencyDetail {
                git: Some(git),
                branch: None,
                ..
            }) => OdraLocation::Remote(git, None),
            _ => {
                Error::FailedToReadCargo("Cargo.toml".to_string()).print_and_die();
            }
        }
    }

    fn odra_location(source: Option<String>) -> OdraLocation {
        // repo
        let source = if let Some(source) = source {
            source
        } else {
            return OdraLocation::Remote(ODRA_TEMPLATE_GH_REPO.to_string(), None);
        };

        // location on disk
        let local = PathBuf::from(&source);
        if local.exists() {
            OdraLocation::Local(local)
        } else {
            // version
            let version_regex = regex::Regex::new(r"^\d+\.\d+\.\d+$").unwrap();
            if version_regex.is_match(&source) {
                OdraLocation::Remote(
                    ODRA_TEMPLATE_GH_REPO.to_string(),
                    Some(format!("release/{}", source)),
                )
            } else {
                // branch
                OdraLocation::Remote(ODRA_TEMPLATE_GH_REPO.to_string(), Some(source))
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum OdraLocation {
    Local(PathBuf),
    /// git repo, branch
    Remote(String, Option<String>),
}

#[derive(Debug, Clone)]
pub struct Member {
    /// Name of the member.
    pub name: String,
    /// Root directory of the member.
    pub root: PathBuf,
    /// Path to the Cargo.toml file.
    pub cargo_toml: PathBuf,
}
