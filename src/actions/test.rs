//! Module responsible for running contracts tests.
use super::build::BuildAction;
use crate::{command, log, project::Project};

/// TestAction configuration.
pub struct TestAction<'a> {
    project: &'a Project,
    backend: Option<String>,
    passthrough_args: Vec<String>,
    skip_build: bool,
    filters: TestFilters,
}

/// TestAction implementation.
impl<'a> TestAction<'a> {
    /// Creates a TestAction struct.
    pub fn new(
        project: &Project,
        backend: Option<String>,
        passthrough_args: Vec<String>,
        skip_build: bool,
        tests: Vec<String>,
        filter: Option<String>,
    ) -> TestAction {
        let filters = TestFilters::new(tests, filter);
        TestAction {
            backend,
            passthrough_args,
            skip_build,
            project,
            filters,
        }
    }
}

impl TestAction<'_> {
    /// Runs a test suite.
    pub fn test(&self) {
        if self.backend.is_none() {
            self.test_odra_vm();
        } else {
            if self.project.odra_toml().contracts.is_empty() {
                log::warn("No contracts found in Odra.toml file.");
            }
            if !self.skip_build {
                self.build_wasm_files();
            }
            self.test_backend();
        }
    }

    /// Test code against OdraVM.
    fn test_odra_vm(&self) {
        log::info("Testing against OdraVM ...");

        command::cargo_test_odra_vm(
            self.project.project_root(),
            &self.filters,
            self.get_passthrough_args(),
        );
    }

    /// Test specific backend.
    fn test_backend(&self) {
        log::info(format!("Testing backend: {}...", self.backend_name()));
        command::cargo_test_backend(
            self.project.project_root(),
            self.backend_name(),
            &self.filters,
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
        BuildAction::new(self.project, None).build();
        log::info("Building finished.")
    }
}

pub struct TestFilters {
    targets: Vec<String>,
    filter: Option<String>,
}

impl TestFilters {
    fn new(targets: Vec<String>, filter: Option<String>) -> Self {
        Self { targets, filter }
    }

    pub fn as_args(&self) -> Vec<&str> {
        let mut args = match &self.targets.len() {
            0 => vec!["--tests"],
            _ => self
                .targets
                .iter()
                .flat_map(|t| vec!["--test", t.as_str()])
                .collect(),
        };

        if let Some(filter) = &self.filter {
            args.push(filter.as_str());
        }

        args
    }
}
