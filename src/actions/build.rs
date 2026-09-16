//! Module for managing and building wasm files.

use crate::{command, errors::Error, log, paths, project::Project, utils};

/// BuildAction configuration.
pub struct BuildAction<'a> {
    contracts_names: Option<String>,
    project: &'a Project,
}

/// BuildAction implementation.
impl<'a> BuildAction<'a> {
    /// Crate a new BuildAction for a given backend.
    pub fn new(project: &'a Project, contracts_names: Option<String>) -> Self {
        BuildAction {
            contracts_names,
            project,
        }
    }
}

impl BuildAction<'_> {
    /// Main function that runs the whole workflow for a backend.
    pub fn build(&self) {
        utils::check_target_requirements();
        utils::validate_contract_name_argument(self.project, self.contracts_names());
        utils::validate_contract_names(self.project);
        self.build_wasm_files();
        self.optimize_wasm_files();
        self.distribute_wasm_files();
    }

    /// Build .wasm files.
    fn build_wasm_files(&self) {
        log::info("Generating wasm files...");
        command::mkdir(paths::wasm_dir(&self.project.project_root()))
            .unwrap_or_else(|err| err.print_and_die());

        let contracts =
            utils::contracts(self.project, self.contracts_names()).unwrap_or_else(|_| {
                Error::FailedToParseArgument("contracts_names".to_string()).print_and_die()
            });

        for contract in contracts {
            let module_name = match self.project.is_workspace() {
                true => contract.module_name(),
                false => contract.crate_name(self.project),
            };
            let build_contract = format!("{}_build_contract", module_name);
            command::cargo_build_wasm_files(
                self.project.project_root(),
                &contract.struct_name(),
                &module_name,
                self.project.is_workspace(),
                contract.module_crate_name(self.project),
            );

            let source = paths::wasm_path_in_target(&build_contract, self.project.project_root());
            let target =
                paths::wasm_path_in_wasm_dir(&contract.struct_name(), &self.project.project_root());
            log::info(format!("Saving {}", target.display()));
            command::cp(source, target);
        }
    }

    /// Copies the optimised wasm files into the `wasm` directory of every workspace member.
    ///
    /// The Casper test VM loads `wasm/<Contract>.wasm` relative to the working directory, and
    /// Cargo runs each member's tests inside that member's directory. Tests that deploy a
    /// contract can live in any member, not only in the crate that defines it, so every
    /// member gets a copy.
    fn distribute_wasm_files(&self) {
        if !self.project.is_workspace() {
            return;
        }
        let contracts =
            utils::contracts(self.project, self.contracts_names()).unwrap_or_else(|_| {
                Error::FailedToParseArgument("contracts_names".to_string()).print_and_die()
            });
        for member_root in &self.project.workspace_members {
            let member_wasm_dir = paths::wasm_dir(member_root);
            command::mkdir(member_wasm_dir.clone()).unwrap_or_else(|err| err.print_and_die());
            for contract in &contracts {
                let source = paths::wasm_path_in_wasm_dir(
                    &contract.struct_name(),
                    &self.project.project_root(),
                );
                let target = paths::wasm_path_in_wasm_dir(&contract.struct_name(), member_root);
                log::info(format!("Copying to {}", target.display()));
                command::cp(source, target);
            }
        }
    }

    /// Run wasm-strip on *.wasm files in wasm directory.
    fn optimize_wasm_files(&self) {
        log::info("Optimizing wasm files...");
        let contracts =
            utils::contracts(self.project, self.contracts_names()).unwrap_or_else(|_| {
                Error::FailedToParseArgument("contracts_names".to_string()).print_and_die()
            });

        for contract in contracts {
            command::process_wasm(&contract.struct_name(), self.project.project_root());
        }
    }

    fn contracts_names(&self) -> String {
        self.contracts_names.clone().unwrap_or_default()
    }
}
