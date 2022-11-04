//! Module responsible for initializing Odra project
use std::path::Path;

use crate::{errors::Error, InitCommand};
use cargo_generate::{GenerateArgs, TemplatePath, Vcs};
use chrono::Utc;
// TODO: remove.
use heck::ToSnakeCase;

/// Init struct
pub struct InitAction {
    name: String,
    repo_uri: String,
}

// TODO: Comments
impl InitAction {
    pub fn new(init: InitCommand) -> InitAction {
        InitAction {
            name: init.name,
            repo_uri: init.repo_uri,
        }
    }

    pub fn generate_project(&self, init: bool) {
        if init {
            self.assert_current_dir_is_empty();
        }
        cargo_generate::generate(GenerateArgs {
            template_path: TemplatePath {
                auto_path: Some(self.repo_uri.clone()),
                subfolder: None,
                git: None,
                branch: None,
                path: None,
                favorite: None,
            },
            list_favorites: false,
            name: Some(self.name.to_snake_case()),
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

    fn assert_current_dir_is_empty(&self) {
        let not_empty = Path::new(".").read_dir().unwrap().next().is_some();
        if not_empty {
            Error::CurrentDirIsNotEmpty.print_and_die();
        }
    }
}
