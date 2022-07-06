use crate::{builder, BuildCommand, TestCommand};
use std::os::unix::process::CommandExt;
use std::process::Command;
pub struct Tests {
    test: TestCommand,
}

impl Tests {
    pub fn new(test: TestCommand) -> Tests {
        Tests { test }
    }

    pub(crate) fn test(&self) {
        match self.test.backend {
            None => {
                self.test_mock_vm();
            }
            Some(_) => {
                self.test_backend();
            }
        }
    }

    fn test_backend(&self) {
        builder::Builder::new(BuildCommand {
            backend: self.test.backend.clone(),
            repo_uri: self.test.repo_uri.clone(),
        })
        .build();

        let mut test_args = self.get_test_args();
        test_args.append(&mut vec!["--no-default-features", "--features=wasm-test"]);

        println!("Running cargo test...");
        Command::new("cargo").args(test_args).exec();
    }

    fn test_mock_vm(&self) {
        let test_args = self.get_test_args();
        Command::new("cargo").args(test_args).exec();
    }

    fn get_test_args(&self) -> Vec<&str> {
        let mut test_args = vec!["test"];
        match &self.test.passthrough {
            None => {}
            Some(passthrough) => {
                let passthrough = passthrough.first().unwrap().as_os_str().to_str().unwrap();
                let mut vec: Vec<&str> = passthrough.split(' ').collect();
                test_args.append(&mut vec);
            }
        }
        test_args
    }
}
