//! Module responsible for running contracts tests.

use super::build::BuildAction;
use crate::{command, paths};

/// TestAction configuration.
pub struct TestAction {
    backend: Option<String>,
    passthrough_args: Vec<String>,
    skip_build: bool,
}

/// TestAction implementation.
impl TestAction {
    /// Creates a TestAction struct.
    pub fn new(
        backend: Option<String>,
        passthrough_args: Vec<String>,
        skip_build: bool,
    ) -> TestAction {
        TestAction {
            backend,
            passthrough_args,
            skip_build,
        }
    }

    /// Runs a test suite.
    pub fn test(&self) {
        if self.backend.is_none() {
            self.test_mock_vm();
        } else {
            if !self.skip_build {
                self.build_wasm_files();
            }
            self.test_backend();
        }
    }

    /// Test code against MockVM.
    fn test_mock_vm(&self) {
        command::cargo_test_mock_vm(paths::project_dir(), self.get_passthrough_args());
    }

    /// Test specific backend.
    fn test_backend(&self) {
        command::cargo_test_backend(
            paths::project_dir(),
            self.backend_name(),
            self.get_passthrough_args(),
        );
    }

    /// Returns backend name.
    fn backend_name(&self) -> &str {
        self.backend.as_ref().unwrap()
    }

    /// Returns passthrough args to be appended at the end of `cargo test` command.
    fn get_passthrough_args(&self) -> Vec<&str> {
        self.passthrough_args.iter().map(AsRef::as_ref).collect()
    }

    /// Build *.wasm files before testing.
    fn build_wasm_files(&self) {
        BuildAction::new(String::from(self.backend_name()), None).build();
    }
}
