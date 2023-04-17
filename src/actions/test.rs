//! Module responsible for running contracts tests.

use std::path::PathBuf;

use super::build::BuildAction;
use crate::{command, log, project::Project};

/// TestAction configuration.
pub struct TestAction {
    backend: Option<String>,
    passthrough_args: Vec<String>,
    skip_build: bool,
    project_root: PathBuf,
}

/// TestAction implementation.
impl TestAction {
    /// Creates a TestAction struct.
    pub fn new(
        backend: Option<String>,
        passthrough_args: Vec<String>,
        skip_build: bool,
        project_root: PathBuf,
    ) -> TestAction {
        TestAction {
            backend,
            passthrough_args,
            skip_build,
            project_root,
        }
    }

    /// Runs a test suite.
    pub fn test(&self) {
        if self.backend.is_none() {
            self.test_mock_vm(self.project_root());
        } else {
            if !self.skip_build {
                self.build_wasm_files();
            }
            self.test_backend();
        }
    }

    /// Test code against MockVM.
    fn test_mock_vm(&self, project_root: PathBuf) {
        log::info("Testing against MockVM ...");
        command::cargo_test_mock_vm(project_root, self.get_passthrough_args());
    }

    /// Test specific backend.
    fn test_backend(&self) {
        log::info(format!("Testing backend: {}...", self.backend_name()));
        command::cargo_test_backend(
            self.project_root(),
            self.backend_name(),
            self.get_passthrough_args(),
        );
    }

    /// Returns backend name.
    fn backend_name(&self) -> &str {
        self.backend.as_ref().unwrap()
    }
    /// Returns project root directory.
    fn project_root(&self) -> PathBuf {
        self.project_root.clone()
    }

    /// Returns passthrough args to be appended at the end of `cargo test` command.
    fn get_passthrough_args(&self) -> Vec<&str> {
        self.passthrough_args.iter().map(AsRef::as_ref).collect()
    }

    /// Build *.wasm files before testing.
    fn build_wasm_files(&self) {
        BuildAction::new(
            Project::detect(Some(self.project_root())),
            String::from(self.backend_name()),
        )
        .build();
        log::info("Building finished.")
    }
}
