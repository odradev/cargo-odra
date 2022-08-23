use crate::{info, Backend, TestCommand};
use std::os::unix::process::CommandExt;
use std::process::Command;

pub struct Tests {
    backend: Option<Backend>,
    passthrough: Vec<String>,
}

impl Tests {
    /// Creates a Test struct
    pub fn new(test: TestCommand) -> Tests {
        let backend = test.backend.map(Backend::load);

        Tests {
            backend,
            passthrough: test.args,
        }
    }

    /// Runs a test suite
    pub fn test(&self) {
        match &self.backend {
            None => {
                self.test_mock_vm();
            }
            Some(_) => {
                self.test_backend();
            }
        }
    }

    fn test_backend(&self) {
        self.backend.clone().unwrap().build();

        let mut test_args = self.get_test_args();
        test_args.append(&mut vec![
            "--no-default-features".to_string(),
            "--features=wasm-test".to_string(),
        ]);

        info("Running cargo test...");
        Command::new("cargo").args(test_args).exec();
    }

    fn test_mock_vm(&self) {
        let mut test_args = self.get_test_args();
        test_args.append(&mut vec![
            "--no-default-features".to_string(),
            "--features=mock-vm".to_string(),
        ]);

        info("Running cargo test...");
        Command::new("cargo").args(test_args).exec();
    }

    fn get_test_args(&self) -> Vec<String> {
        let mut test_args = vec!["test".to_string()];
        if !self.passthrough.is_empty() {
            test_args.append(&mut self.passthrough.clone());
        }

        test_args
    }
}
