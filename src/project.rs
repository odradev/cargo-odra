use std::{
    env,
    path::{Path, PathBuf},
};

use cargo_generate::{GenerateArgs, TemplatePath, Vcs};
use chrono::Utc;

use crate::{
    actions::{build::BuildAction, generate::GenerateAction, init::InitAction, test::TestAction},
    cargo_toml::load_cargo_toml,
    cli::{GenerateCommand, TestCommand},
    errors::Error,
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
    /// Branch of Odra to use
    pub branch: String,
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
            Self::assert_current_dir_is_empty();
        }

        cargo_generate::generate(GenerateArgs {
            template_path: TemplatePath {
                auto_path: Some(init_action.repo_uri),
                subfolder: Some(format!("templates/{}", init_action.template)),
                test: false,
                git: None,
                branch: Some(init_action.branch.clone()),
                tag: None,
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
        .unwrap();
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
        let members = Self::find_members(cargo_toml_path.clone());
        let name = match load_cargo_toml(cargo_toml_path.clone()).package {
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
            // todo: get branch from Odra.toml
            branch: "feature/cargo_odra_templates".to_string(),
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
        GenerateAction::new(
            self,
            generate.contract_name,
            generate.module,
            generate.git_branch,
        )
        .generate_contract();
    }

    /// Odra.toml location for the Project.
    pub fn odra_toml_location(&self) -> PathBuf {
        self.odra_toml_location.clone()
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

    // todo: get branch from Cargo.toml, or even better - use Dependency
    pub fn branch(&self) -> String {
        "feature/cargo-odra-templates".to_string()
    }

    /// Check if the project is a workspace.
    pub fn is_workspace(&self) -> bool {
        self.members.len() > 1 || self.members[0].root != self.project_root()
    }

    fn assert_current_dir_is_empty() {
        let not_empty = Path::new(".").read_dir().unwrap().next().is_some();
        if not_empty {
            Error::CurrentDirIsNotEmpty.print_and_die();
        }
    }

    fn find_members(cargo_toml_path: PathBuf) -> Vec<Member> {
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

    fn detect_members(cargo_toml_path: PathBuf) -> Vec<(String, String)> {
        match load_cargo_toml(cargo_toml_path.clone()).workspace {
            Some(workspace) => workspace
                .members
                .iter()
                .map(|member| (member.clone(), member.clone()))
                .collect(),
            None => vec![(Self::detect_project_name(cargo_toml_path), "".to_string())],
        }
    }

    fn detect_project_name(cargo_toml_path: PathBuf) -> String {
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
