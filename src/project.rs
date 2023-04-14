use std::env;

use crate::actions::generate::GenerateAction;
use crate::actions::init::InitAction;
use crate::actions::test::TestAction;
use crate::cargo_toml::{load_cargo_toml, load_main_cargo_toml};
use crate::cli::{GenerateCommand, TestCommand};
use crate::errors::Error;
use crate::paths;
use cargo_generate::{GenerateArgs, TemplatePath, Vcs};
use chrono::Utc;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct Project {
    /// Name of the project.
    pub name: String,
    /// Root directory of the project.
    pub root: PathBuf,
    /// Path to the main Cargo.toml file.
    pub cargo_toml: PathBuf,
    /// Path to the Odra.toml file.
    pub odra_toml: PathBuf,
    /// Members of the project.
    pub members: Vec<Member>,
}

impl Project {
    pub fn new(init_action: InitAction) {
        let current_dir = env::current_dir().unwrap();
        if init_action.generate {
            Self::generate_project(init_action.clone());
        }
    }

    /// Make sure current dir is empty.
    fn assert_current_dir_is_empty() {
        let not_empty = Path::new(".").read_dir().unwrap().next().is_some();
        if not_empty {
            Error::CurrentDirIsNotEmpty.print_and_die();
        }
    }

    pub fn generate_project(init_action: InitAction) {
        if init_action.init {
            Self::assert_current_dir_is_empty();
        }

        cargo_generate::generate(GenerateArgs {
            template_path: TemplatePath {
                auto_path: Some(init_action.repo_uri.clone()),
                subfolder: None,
                git: None,
                branch: Some(init_action.branch.clone()),
                path: None,
                favorite: None,
            },
            list_favorites: false,
            name: Some(paths::to_snake_case(&init_action.project_name)),
            force: true,
            verbose: false,
            template_values_file: None,
            silent: false,
            config: None,
            vcs: Vcs::Git,
            lib: false,
            bin: false,
            ssh_identity: None,
            define: vec![format!("date={}", Utc::now().format("%Y-%m-%d"))],
            init: init_action.init,
            destination: None,
            force_git_init: false,
            allow_commands: false,
        })
        .unwrap();
    }

    pub fn detect(path: Option<PathBuf>) -> Project {
        let odra_toml_path = Self::find_odra_toml(path.clone()).unwrap_or_else(|| {
            Error::NotAnOdraProject.print_and_die();
        });
        let cargo_toml_path = Self::find_cargo_toml(path).unwrap_or_else(|| {
            Error::NotAnOdraProject.print_and_die();
        });
        let root = odra_toml_path.parent().unwrap().to_path_buf();
        let members = Self::find_members(cargo_toml_path.clone());
        let name = match load_main_cargo_toml().package {
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
            root,
            cargo_toml: cargo_toml_path,
            odra_toml: odra_toml_path,
            members,
        }
    }

    pub fn test(&self, test: TestCommand) {
        TestAction::new(test.backend, test.args, test.skip_build).test();
    }

    pub fn generate(&self, generate: GenerateCommand) {
        dbg!(self.members.clone());
        let (module_path, module_name) = match generate.module {
            None => match self.is_workspace() {
                true => {
                    Error::ModuleNotSpecified.print_and_die();
                }
                false => (
                    self.root.clone(),
                    self.members.first().unwrap().name.clone(),
                ),
            },
            Some(module) => {
                let module_name = module.clone();
                let module_path = self
                    .members
                    .iter()
                    .find(|member| member.name == module)
                    .unwrap_or_else(|| {
                        Error::ModuleNotFound(module).print_and_die();
                    })
                    .root
                    .clone();
                (module_path, module_name)
            }
        };
        let module_path = module_path
            .strip_prefix(self.root.clone())
            .unwrap()
            .to_path_buf();
        GenerateAction::new(generate.contract_name, module_path, module_name).generate_contract();
    }

    pub fn is_workspace(&self) -> bool {
        let cargo_toml = load_main_cargo_toml();
        cargo_toml.workspace.is_some()
    }

    pub fn find_members(cargo_toml_path: PathBuf) -> Vec<Member> {
        Self::detect_members(cargo_toml_path.clone())
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
    pub fn find_odra_toml(path: Option<PathBuf>) -> Option<PathBuf> {
        Self::find_file_upwards("Odra.toml", path)
    }

    pub fn find_cargo_toml(path: Option<PathBuf>) -> Option<PathBuf> {
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

    fn find_file_upwards(filename: &str, path: Option<PathBuf>) -> Option<PathBuf> {
        let mut path = path
            .unwrap_or_else(|| std::env::current_dir().expect("Failed to get current directory"));
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

    pub fn detect_members(cargo_toml_path: PathBuf) -> Vec<(String, String)> {
        dbg!(cargo_toml_path.clone());
        match load_cargo_toml(cargo_toml_path.clone()).workspace {
            Some(workspace) => workspace
                .members
                .iter()
                .map(|member| (member.clone(), member.clone()))
                .collect(),
            None => vec![(Self::detect_project_name(cargo_toml_path), "".to_string())],
        }
    }

    /// Returns project's name from Cargo.toml.
    pub fn detect_project_name(cargo_toml_path: PathBuf) -> String {
        load_cargo_toml(cargo_toml_path).package.unwrap().name
    }
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
