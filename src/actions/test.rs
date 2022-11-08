//! Module responsible for running contracts tests
use super::build::BuildAction;
use crate::{cli::TestCommand, command, paths};

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
        command::cargo_test_mock_vm(paths::project_dir(), self.get_passthrough_args());
    }

    fn test_backend(&self) {
        command::cargo_test_backend(
            paths::project_dir(),
            self.backend_name(),
            self.get_passthrough_args(),
        );
    }

    fn backend_name(&self) -> &str {
        self.backend.as_ref().unwrap()
    }

    fn get_passthrough_args<'a>(&'a self) -> Vec<&'a str> {
        if !self.passthrough.is_empty() {
            self.passthrough.iter().map(|s| s.as_ref()).collect()
        } else {
            vec![]
        }
    }

    fn build_wasm_files(&self) {
        BuildAction::new(String::from(self.backend_name())).build();
    }
}
