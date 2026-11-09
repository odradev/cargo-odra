//! Module for managing and building wasm files.

use std::{collections::VecDeque, path::Path, sync::Mutex, thread};

use crate::{
    actions::schema::SchemaAction,
    command::{self, WasmOptFlags},
    errors::Error,
    log,
    odra_toml::Contract,
    paths,
    project::Project,
    utils,
};

/// Most contracts built at the same time when `--jobs` is not given.
const MAX_DEFAULT_JOBS: usize = 8;

/// BuildAction configuration.
pub struct BuildAction<'a> {
    contracts_names: Option<String>,
    skip_schema: bool,
    jobs: usize,
    project: &'a Project,
}

/// BuildAction implementation.
impl<'a> BuildAction<'a> {
    /// Crate a new BuildAction for a given backend.
    ///
    /// `jobs` is how many contracts are built at the same time; without it, see [`default_jobs`].
    pub fn new(
        project: &'a Project,
        contracts_names: Option<String>,
        skip_schema: bool,
        jobs: Option<usize>,
    ) -> Self {
        let jobs = jobs.unwrap_or_else(|| {
            default_jobs(thread::available_parallelism().map_or(1, |cpus| cpus.get()))
        });
        BuildAction {
            contracts_names,
            skip_schema,
            jobs: jobs.max(1),
            project,
        }
    }
}

/// How many contracts to build at the same time on a machine with `cpus` CPUs when the user does
/// not say: a quarter of them, at least one and at most [`MAX_DEFAULT_JOBS`].
///
/// Every contract is a separate `cargo build` of the same crate, since the contract is chosen
/// with a cfg flag, and each keeps about one CPU busy: the crate compiles with a single codegen
/// unit and links with fat LTO. Builds could therefore run one per CPU, but every job past the
/// first owns a target directory with its own copy of the dependencies, a few hundred MB that
/// are compiled once more on first use, so the default leaves room rather than taking every core.
fn default_jobs(cpus: usize) -> usize {
    (cpus / 4).clamp(1, MAX_DEFAULT_JOBS)
}

impl BuildAction<'_> {
    /// Main function that runs the whole workflow for a backend.
    pub fn build(&self) {
        utils::check_target_requirements();
        utils::validate_contract_name_argument(self.project, self.contracts_names());
        utils::validate_contract_names(self.project);
        if !self.skip_schema {
            SchemaAction::new(self.project, self.contracts_names.clone()).build();
        }
        self.build_wasm_files();
    }

    /// Builds, saves and optimizes the wasm file of every selected contract, `jobs` at a time.
    ///
    /// The contracts wait in a queue that the workers take from, so a slow contract does not hold
    /// up the others. Worker 0 builds in the project's target directory, like a single job always
    /// did; the others in a target directory of their own, since Cargo locks a target directory
    /// for the duration of a build. The first failure stops the workers once each has finished
    /// the build it is on, and is reported after they have.
    fn build_wasm_files(&self) {
        log::info("Generating wasm files...");
        let project_root = self.project.project_root();
        command::mkdir(paths::wasm_dir(&project_root)).unwrap_or_else(|err| err.print_and_die());

        let contracts =
            utils::contracts(self.project, self.contracts_names()).unwrap_or_else(|_| {
                Error::FailedToParseArgument("contracts_names".to_string()).print_and_die()
            });

        utils::validate_external_contracts(self.project, &contracts);

        // Checked before the first build, so a missing wasm-opt is not found out after the last.
        let wasm_opt = WasmOptFlags::detect();
        let target_dir = paths::build_target_dir(&project_root);
        let jobs = self.jobs.min(contracts.len()).max(1);
        if jobs > 1 {
            log::info(format!(
                "Building {} contracts, {jobs} at a time...",
                contracts.len()
            ));
        }

        let queue = Mutex::new(VecDeque::from(contracts));
        let failure: Mutex<Option<Error>> = Mutex::new(None);
        thread::scope(|scope| {
            for worker in 0..jobs {
                let worker_dir = paths::worker_target_dir(&target_dir, worker);
                let (queue, failure, target_dir, wasm_opt) =
                    (&queue, &failure, &target_dir, &wasm_opt);
                scope.spawn(move || {
                    if let Some(dir) = worker_dir.as_deref().filter(|dir| !dir.exists()) {
                        log::info(format!(
                            "Build directory {} is new; its first build compiles the dependencies.",
                            dir.display()
                        ));
                    }
                    loop {
                        if failure.lock().unwrap().is_some() {
                            return;
                        }
                        let Some(contract) = queue.lock().unwrap().pop_front() else {
                            return;
                        };
                        let result = self.build_contract(
                            &contract,
                            worker_dir.as_deref().unwrap_or(target_dir),
                            worker_dir.as_deref(),
                            jobs > 1,
                            wasm_opt,
                        );
                        if let Err(error) = result {
                            *failure.lock().unwrap() = Some(error);
                            return;
                        }
                    }
                });
            }
        });
        if let Some(error) = failure.into_inner().unwrap() {
            error.print_and_die();
        }
    }

    /// Builds one contract, saves its wasm to the `wasm` directory and optimizes it.
    ///
    /// `target_dir` is where Cargo puts the build; it is passed to Cargo as `worker_dir` unless
    /// it is the project's own target directory. With `capture_output`, Cargo's output is written
    /// out in one piece once the build is over, so parallel builds do not interleave.
    fn build_contract(
        &self,
        contract: &Contract,
        target_dir: &Path,
        worker_dir: Option<&Path>,
        capture_output: bool,
        wasm_opt: &WasmOptFlags,
    ) -> Result<(), Error> {
        let project_root = self.project.project_root();
        let host_crate = contract.host_crate(self.project);
        command::cargo_build_wasm_files(
            &project_root,
            &contract.odra_module_value(self.project),
            &host_crate,
            self.project.is_cargo_workspace(),
            worker_dir,
            capture_output,
        )?;

        let build_contract = format!("{}_build_contract", host_crate.name);
        let source = paths::wasm_path_in_target_dir(target_dir, &build_contract);
        let target = paths::wasm_path_in_wasm_dir(&contract.struct_name(), &project_root);
        log::info(format!("Saving {}", target.display()));
        command::copy_file(&source, &target)?;
        self.remove_stale_member_copies(contract);
        command::process_wasm(&contract.struct_name(), &project_root, wasm_opt)
    }

    /// Removes this contract's wasm from every workspace member.
    ///
    /// `cargo-odra` 0.1.x copied the built file into the crate that defines the contract. The
    /// Casper test VM takes the first `wasm/<Contract>.wasm` it finds walking up from the
    /// working directory, so a copy left behind by an older version shadows the file just
    /// written at the project root, and the tests of that member would keep running the old
    /// bytecode. Only files this command would itself produce are removed.
    fn remove_stale_member_copies(&self, contract: &Contract) {
        for member_root in &self.project.workspace_members {
            let stale = paths::wasm_path_in_wasm_dir(&contract.struct_name(), member_root);
            if stale.is_file() {
                log::info(format!("Removing stale {}", stale.display()));
                command::rm_file(stale);
            }
        }
    }

    fn contracts_names(&self) -> String {
        self.contracts_names.clone().unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::{default_jobs, MAX_DEFAULT_JOBS};

    #[test]
    fn default_jobs_is_a_quarter_of_the_cpus_within_bounds() {
        // Small machines and CI runners build one contract at a time, as before.
        assert_eq!(default_jobs(1), 1);
        assert_eq!(default_jobs(4), 1);
        assert_eq!(default_jobs(7), 1);
        assert_eq!(default_jobs(8), 2);
        assert_eq!(default_jobs(16), 4);
        assert_eq!(default_jobs(32), MAX_DEFAULT_JOBS);
        assert_eq!(default_jobs(128), MAX_DEFAULT_JOBS);
    }
}
