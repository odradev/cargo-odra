//! Module responsible for running contracts tests.
use super::build::BuildAction;
use crate::{command, log, project::Project};

/// TestAction configuration.
pub struct TestAction<'a> {
    project: &'a Project,
    backend: Option<String>,
    passthrough_args: Vec<String>,
    skip_build: bool,
    jobs: Option<usize>,
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
        jobs: Option<usize>,
    ) -> TestAction<'_> {
        let filters = TestFilters::new(tests, filter);
        TestAction {
            backend,
            passthrough_args,
            skip_build,
            jobs,
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
        cargo_test_tail(&self.passthrough_args)
    }

    /// Build *.wasm files before testing.
    fn build_wasm_files(&self) {
        BuildAction::new(self.project, None, true, self.jobs).build();
        log::info("Building finished.")
    }
}

/// What follows `cargo test [filters]` for the arguments given after `cargo odra test --`.
///
/// They go to the test binary, as they do after `cargo test --`: `cargo odra test -- --nocapture`
/// runs `cargo test .. -- --nocapture`. Arguments for Cargo itself come before a second `--`:
/// `cargo odra test -- --release -- --nocapture` runs `cargo test .. --release -- --nocapture`.
fn cargo_test_tail(passthrough_args: &[String]) -> Vec<&str> {
    let args: Vec<&str> = passthrough_args.iter().map(AsRef::as_ref).collect();
    if args.is_empty() || args.contains(&"--") {
        return args;
    }
    let mut tail = vec!["--"];
    tail.extend(args);
    tail
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

#[cfg(test)]
mod tests {
    use super::cargo_test_tail;

    fn tail(args: &[&str]) -> Vec<String> {
        let args: Vec<String> = args.iter().map(|arg| arg.to_string()).collect();
        cargo_test_tail(&args)
            .into_iter()
            .map(str::to_string)
            .collect()
    }

    #[test]
    fn passthrough_args_go_to_the_test_binary() {
        assert!(tail(&[]).is_empty());
        assert_eq!(tail(&["--nocapture"]), ["--", "--nocapture"]);
        assert_eq!(
            tail(&["two", "--test-threads", "1"]),
            ["--", "two", "--test-threads", "1"]
        );
    }

    #[test]
    fn a_second_separator_splits_cargo_args_from_test_binary_args() {
        assert_eq!(
            tail(&["--release", "--", "--nocapture"]),
            ["--release", "--", "--nocapture"]
        );
        assert_eq!(tail(&["--release", "--"]), ["--release", "--"]);
        // What worked before the arguments went to the test binary still does.
        assert_eq!(tail(&["--", "--nocapture"]), ["--", "--nocapture"]);
    }
}
