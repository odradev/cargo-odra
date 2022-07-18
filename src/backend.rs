use crate::command::parse_command_result;
use std::fs;
use std::path::Path;
use std::process::Command;

pub struct Backend {
    name: String,
    repo_uri: String,
}

impl Backend {
    pub fn new(name: String, repo_uri: Option<String>) -> Backend {
        let uri = match repo_uri {
            None => {
                format!("https://github.com/odradev/odra-{}.git", name)
            }
            Some(repo_uri) => repo_uri,
        };

        Backend {
            name,
            repo_uri: uri,
        }
    }

    pub fn repo_uri(&self) -> &String {
        &self.repo_uri
    }

    pub fn path(&self) -> String {
        format!(".backend_{}/", self.name)
    }
    pub fn name(&self) -> &String {
        &self.name
    }
    pub fn test_env_path(&self) -> String {
        format!("{}test_env/", self.path())
    }

    pub(crate) fn pull_backend(&self) {
        if !Path::new(self.path().as_str()).is_dir() {
            println!("Downloading repository from {}...", self.repo_uri);
            let command = Command::new("git")
                .args(vec!["clone", self.repo_uri.as_str(), self.path().as_str()])
                .status()
                .unwrap();

            parse_command_result(command, "Couldn't pull repository");
        }
    }

    pub(crate) fn build_backend(&self) {
        println!("Building {} backend...", self.name);
        let command = Command::new("cargo")
            .current_dir(self.test_env_path())
            .args(vec!["build"])
            .status()
            .expect("Couldn't build backend");

        parse_command_result(command, "Couldn't build backend");

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
