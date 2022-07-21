use crate::command::parse_command_result;
use std::fs;
use std::path::Path;
use std::process::Command;

pub struct Backend {
    name: String,
    path: String,
}

impl Backend {
    pub fn new(name: String, path: Option<String>) -> Backend {
        Backend {
            name: name.clone(),
            path: match path {
                None => {
                    format!("https://github.com/odradev/odra-{}.git", name)
                }
                Some(path) => path,
            },
        }
    }

    pub fn path(&self) -> &String {
        &self.path
    }
    pub fn backend_path(&self) -> String {
        format!(".backend_{}/", self.name)
    }
    pub fn name(&self) -> &String {
        &self.name
    }
    pub fn test_env_path(&self) -> String {
        format!("{}test_env/", self.backend_path())
    }

    fn pull_backend(&self) {
        if !Path::new(self.backend_path().as_str()).is_dir() {
            println!("Downloading repository from {}...", self.path);
            let command = Command::new("git")
                .args(vec![
                    "clone",
                    "--branch",
                    "develop",
                    self.path.as_str(),
                    self.backend_path().as_str()
                ])
                .status()
                .unwrap();

            parse_command_result(command, "Couldn't pull repository");
        }
    }

    pub(crate) fn build_backend(&self) {
        self.pull_backend();
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
