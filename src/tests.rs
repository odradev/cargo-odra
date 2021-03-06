use crate::{BuildCommand, Builder, TestCommand};
use std::ffi::OsString;
use std::os::unix::process::CommandExt;
use std::process::Command;
pub struct Tests {
    builder: Builder,
    passthrough: Option<Vec<OsString>>,
}

impl Tests {
    pub fn new(test: TestCommand) -> Tests {
        let builder = Builder::new(BuildCommand {
            backend: test.backend,
            repo_uri: test.repo_uri,
        });
        Tests {
            builder,
            passthrough: test.passthrough,
        }
    }

    pub(crate) fn test(&self) {
        match self.builder.backend {
            None => {
                self.test_mock_vm();
            }
            Some(_) => {
                self.test_backend();
            }
        }
    }

    fn test_backend(&self) {
        self.builder.build();

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
        match &self.passthrough {
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
