use std::fs;
use std::path::Path;
use std::process::Command;

pub(crate) struct Backend {
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

    pub fn path(&self) -> String {
        format!(".backend_{}/", self.name)
    }
    pub fn test_env_path(&self) -> String {
        format!("{}test_env/", self.path())
    }

    pub(crate) fn pull_backend(&self) {
        if !Path::new(self.path().as_str()).is_dir() {
            println!("Downloading repository from {}...", self.repo_uri);
            Command::new("git")
                .args(vec!["clone", self.repo_uri.as_str(), self.path().as_str()])
                .output()
                .unwrap();
        }
    }

    pub(crate) fn build_backend(&self) {
        if Path::new("target/debug/libodra_test_env.so").exists() {
            return;
        }

        println!("Building {} backend...", self.name);
        Command::new("cargo")
            .current_dir(self.test_env_path())
            .args(vec!["build"])
            .output()
            .expect("Couldn't build backend");
        println!("Copying lib...");
        fs::create_dir_all("./target/debug").unwrap();

        let source = format!("{}target/debug/libodra_test_env.so", self.test_env_path());
        let target = "./target/debug/libodra_test_env.so";

        Command::new("cp")
            .args(vec![source, target.to_string()])
            .output()
            .expect("Couldn't copy lib");
    }
}
