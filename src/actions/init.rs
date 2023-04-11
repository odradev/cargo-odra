//! Module responsible for initializing an Odra project.

use std::fs::File;
use std::io::Write;
use std::path::Path;

use cargo_generate::{GenerateArgs, TemplatePath, Vcs};
use chrono::Utc;

use crate::errors::Error::NotAnOdraProject;
use crate::project::Member;
use crate::odra_toml::OdraToml;
use crate::template::WORKSPACE_CARGO_TOML;
use crate::{errors::Error, odra_toml, paths};

/// InitAction configuration.
pub struct InitAction {
    contract_name: String,
    repo_uri: String,
    branch: String,
}

/// WorkspaceInitAction configuration.
pub struct WorkspaceInitAction {}

/// InitAction implementation.
impl InitAction {
    /// Crate a new InitAction.
    pub fn new(contract_name: String, repo_uri: String, branch: String) -> InitAction {
        InitAction {
            contract_name,
            repo_uri,
            branch,
        }
    }

    /// Generate a new project.
    pub fn generate_project(&self, init: bool) {
        if init {
            Self::assert_current_dir_is_empty();
        }

        if let Some(odra_toml) = OdraToml::load() {
            if !odra_toml.is_workspace() {
                NotAnOdraProject.print_and_die();
            }
        }

        cargo_generate::generate(GenerateArgs {
            template_path: TemplatePath {
                auto_path: Some(self.repo_uri.clone()),
                subfolder: None,
                git: None,
                branch: Some(self.branch.clone()),
                path: None,
                favorite: None,
            },
            list_favorites: false,
            name: Some(paths::to_snake_case(&self.contract_name)),
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
            init,
            destination: None,
            force_git_init: false,
            allow_commands: false,
        })
        .unwrap();
    }

    /// Make sure current dir is empty.
    fn assert_current_dir_is_empty() {
        let not_empty = Path::new(".").read_dir().unwrap().next().is_some();
        if not_empty {
            Error::CurrentDirIsNotEmpty.print_and_die();
        }
    }
}

/// WorkspaceInitAction implementation.
impl WorkspaceInitAction {
    /// Crate a new WorkspaceInitAction.
    pub fn new() -> WorkspaceInitAction {
        WorkspaceInitAction {}
    }

    /// Initialize a new workspace.
    pub fn init_workspace(&self) {
        InitAction::assert_current_dir_is_empty();
        let mut file = File::create("Cargo.toml").unwrap();
        file.write_all(WORKSPACE_CARGO_TOML.as_bytes()).unwrap();

        // Create Odra.toml file
        OdraToml::new().save();
    }
}
