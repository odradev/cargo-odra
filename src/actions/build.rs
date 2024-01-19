//! Module for managing and building backends.

use super::utils;
use crate::{command, log, paths, project::Project};

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
        self.build_wasm_files();
        self.optimize_wasm_files();
    }

    /// Build .wasm files.
    fn build_wasm_files(&self) {
        log::info("Generating wasm files...");
        command::mkdir(paths::wasm_dir(self.project.project_root()));

        let contracts =
            utils::contracts(self.project, self.contracts_names()).unwrap_or_else(|_| {
                Error::FailedToParseArgument("contracts_names".to_string()).print_and_die()
            });

        for contract in contracts {
            let build_contract = format!("{}_build_contract", &contract.module_name());
            command::cargo_build_wasm_files(
                self.project.project_root(),
                &contract.struct_name(),
                &contract.module_name(),
            );
            let source = paths::wasm_path_in_target(&build_contract, self.project.project_root());
            let target =
                paths::wasm_path_in_wasm_dir(&contract.struct_name(), self.project.project_root());
            log::info(format!("Saving {}", target.display()));
            command::cp(source.clone(), target);
            // if it's a workspace, copy the file also to the module wasm folder
            if self.project.is_workspace() {
                let module_wasm_dir = self
                    .project
                    .project_root()
                    .join(contract.module_name())
                    .join("wasm");
                command::mkdir(module_wasm_dir.clone());
                let mut module_wasm_path = module_wasm_dir.clone().join(&contract.struct_name());
                module_wasm_path.set_extension("wasm");
                log::info(format!("Copying to {}", module_wasm_path.display()));
                command::cp(source, module_wasm_path);
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
            command::wasm_strip(&contract.struct_name(), self.project.project_root());
            if self.project.is_workspace() {
                command::wasm_strip(
                    &contract.struct_name(),
                    self.project.project_root().join(contract.module_name()),
                );
            }
        }
    }

    fn contracts_names(&self) -> String {
        self.contracts_names.clone().unwrap_or_default()
    }
}
