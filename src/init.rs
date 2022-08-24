//! Module responsible for initializing Odra project
use crate::InitCommand;
use cargo_generate::{GenerateArgs, TemplatePath, Vcs};
use chrono::{DateTime, Utc};
use heck::ToSnakeCase;

/// Init struct
pub struct Init {
    init: InitCommand,
}

impl Init {
    pub fn new(mut init: InitCommand) -> Init {
        match init.repo_uri {
            None => init.repo_uri = Some("odradev/odra-template".to_string()),
            Some(_) => {}
        }
        Init { init }
    }

    pub fn generate_project(&self, init: bool) {
        let now: DateTime<Utc> = Utc::now();
        let date = now.format("%Y-%m-%d");
        let name = Some(self.init.name.to_snake_case());
        cargo_generate::generate(GenerateArgs {
            template_path: TemplatePath {
                auto_path: self.init.repo_uri.clone(),
                subfolder: None,
                git: None,
                branch: None,
                path: None,
                favorite: None,
            },
            list_favorites: false,
            name,
            force: true,
            verbose: false,
            template_values_file: None,
            silent: false,
            config: None,
            vcs: Vcs::Git,
            lib: false,
            bin: false,
            ssh_identity: None,
            define: vec![format!("date={}", date)],
            init,
            destination: None,
            force_git_init: false,
            allow_commands: false,
        })
        .unwrap();
    }
}
