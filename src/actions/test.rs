//! Module responsible for running contracts tests
use std::os::unix::process::CommandExt;
use std::process::Command;

use crate::{log, TestCommand};

use super::build::BuildAction;

/// Tester configuration
pub struct TestAction {
    backend: Option<String>,
    passthrough: Vec<String>,
}

impl TestAction {
    /// Creates a Test struct
    pub fn new(test: TestCommand) -> TestAction {
        TestAction {
            backend: test.backend,
            passthrough: test.args,
        }
    }

    /// Runs a test suite
    pub fn test(&self) {
        if self.backend.is_none() {
            self.test_mock_vm();
        } else {
            self.build_wasm_files();
            self.test_backend();
        }
    }

    fn test_mock_vm(&self) {
        let test_args = self.get_test_args();
        log::info("Running cargo test...");
        // TODO: Move to commnads.
        Command::new("cargo").args(test_args).exec();
    }

    fn test_backend(&self) {
        // TODO: Simplify.
        let mut test_args = self.get_test_args();
        test_args.push(String::from("--no-default-features"));
        test_args.push(String::from("--features"));
        test_args.push(self.backend_name());

        // TODO: Move to commnads.
        log::info("Running cargo test...");
        Command::new("cargo").args(test_args).exec();
    }

    fn backend_name(&self) -> String {
        self.backend.clone().unwrap()
    }

    fn get_test_args(&self) -> Vec<String> {
        let mut test_args = vec!["test".to_string()];
        if !self.passthrough.is_empty() {
            test_args.append(&mut self.passthrough.clone());
        }

        test_args
    }

    fn build_wasm_files(&self) {
        BuildAction::new(self.backend_name()).build();
    }
}
