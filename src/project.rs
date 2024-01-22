use std::{
    env,
    path::{Path, PathBuf},
};

use cargo_toml::{Dependency, DependencyDetail};

use crate::{cargo_toml::load_cargo_toml, errors::Error, odra_toml::OdraToml};

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
    /// Detects an existing project.
    pub fn detect(path: PathBuf) -> Project {
        let odra_toml_path = Self::find_odra_toml(path.clone()).unwrap_or_else(|| {
            Error::NotAnOdraProject.print_and_die();
        });
        let cargo_toml_path = Self::find_cargo_toml(path).unwrap_or_else(|| {
            Error::NotAnOdraProject.print_and_die();
        });
        let root = odra_toml_path.parent().unwrap().to_path_buf();
        let members = Self::members(&cargo_toml_path, &odra_toml_path);
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

    /// Checks if the project is a workspace.
    pub fn is_workspace(&self) -> bool {
        !self.members.is_empty()
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

    /// Returns project's OdraToml.
    pub fn odra_toml(&self) -> OdraToml {
        OdraToml::load(&self.odra_toml_location)
    }

    pub fn members(cargo_toml_path: &PathBuf, odra_toml_path: &Path) -> Vec<Member> {
        Self::detect_members(cargo_toml_path, odra_toml_path)
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

    fn detect_members(cargo_toml_path: &PathBuf, odra_toml_path: &Path) -> Vec<(String, String)> {
        let odra_toml = OdraToml::load(odra_toml_path);
        match load_cargo_toml(cargo_toml_path).workspace {
            Some(workspace) => workspace
                .members
                .iter()
                .filter(|member| odra_toml.has_module(member))
                .map(|member| (member.clone(), member.clone()))
                .collect(),
            None => vec![],
        }
    }

    pub fn project_odra_location(&self) -> OdraLocation {
        let cargo_toml = load_cargo_toml(&self.cargo_toml_location);
        let dependencies = match cargo_toml.workspace {
            None => cargo_toml.dependencies,
            Some(workspace) => workspace.dependencies,
        };

        let odra_dependency = dependencies
            .iter()
            .find(|dependency| dependency.0 == "odra")
            .unwrap_or_else(|| Error::OdraNotADependency.print_and_die())
            .1
            .clone();

        match odra_dependency {
            Dependency::Detailed(DependencyDetail {
                version: Some(version),
                git: None,
                ..
            }) => OdraLocation::CratesIO(version),
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
                Error::FailedToReadCargo("Unsupported location of Odra.".to_string())
                    .print_and_die();
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum OdraLocation {
    Local(PathBuf),
    /// git repo, branch
    Remote(String, Option<String>),
    CratesIO(String),
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
