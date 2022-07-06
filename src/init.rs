use crate::InitCommand;
use cargo_generate::{GenerateArgs, TemplatePath, Vcs};

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

    pub(crate) fn new_project(&self) {
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
            name: self.init.name.clone(),
            force: false,
            verbose: false,
            template_values_file: None,
            silent: false,
            config: None,
            vcs: Vcs::Git,
            lib: false,
            bin: false,
            ssh_identity: None,
            define: vec![],
            init: false,
            destination: None,
            force_git_init: false,
            allow_commands: false,
        })
        .unwrap();
    }

    pub(crate) fn init_project(&self) {
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
            name: None,
            force: false,
            verbose: false,
            template_values_file: None,
            silent: false,
            config: None,
            vcs: Vcs::Git,
            lib: false,
            bin: false,
            ssh_identity: None,
            define: vec![],
            init: true,
            destination: None,
            force_git_init: false,
            allow_commands: false,
        })
        .unwrap();
    }
}
