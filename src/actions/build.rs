//! Module for managing and building backends.

use crate::{command, errors::Error, log, odra_toml::Contract, paths, project::Project};

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
        self.check_target_requirements();
        self.validate_contract_name_argument();
        self.build_wasm_files();
        self.optimize_wasm_files();
    }

    /// Returns list of contract to process.
    fn contracts(&self) -> Vec<Contract> {
        let names = self.parse_contracts_names();
        let odra_toml = self.project.odra_toml();
        match names.is_empty() {
            true => odra_toml.contracts,
            false => odra_toml
                .contracts
                .into_iter()
                .filter(|c| names.contains(&c.name))
                .collect(),
        }
    }

    /// Check if wasm32-unknown-unknown target is installed.
    fn check_target_requirements(&self) {
        if !command::command_output("rustup target list --installed")
            .contains("wasm32-unknown-unknown")
        {
            Error::WasmTargetNotInstalled.print_and_die();
        }
    }

    /// Check if contract name argument is valid if set.
    fn validate_contract_name_argument(&self) {
        let names = self.parse_contracts_names();
        names.iter().for_each(|contract_name| {
            if !self
                .project
                .odra_toml()
                .contracts
                .iter()
                .any(|c| c.name == *contract_name)
            {
                Error::ContractNotFound(contract_name.clone()).print_and_die();
            }
        });
    }

    /// Build .wasm files.
    fn build_wasm_files(&self) {
        log::info("Generating wasm files...");
        command::mkdir(paths::wasm_dir(self.project.project_root()));
        for contract in self.contracts() {
            let build_contract = format!("{}_build_contract", &contract.module_name());
            command::cargo_build_wasm_files(
                self.project.project_root(),
                &contract.name,
                &contract.module_name(),
            );
            let source = paths::wasm_path_in_target(&build_contract, self.project.project_root());
            let target = paths::wasm_path_in_wasm_dir(&contract.name, self.project.project_root());
            log::info(format!("Saving {}", target.display()));
            command::cp(source.clone(), target);
            // if a contract is in a module, copy the file also to the module wasm folder
            if self.project.odra_toml().has_module(&contract.module_name())
                && contract.module_name() != self.project.name
            {
                let module_wasm_dir = self
                    .project
                    .project_root()
                    .join(contract.module_name())
                    .join("wasm");
                command::mkdir(module_wasm_dir.clone());
                let mut module_wasm_path = module_wasm_dir.clone().join(&contract.name);
                module_wasm_path.set_extension("wasm");
                log::info(format!("Copying to {}", module_wasm_path.display()));
                command::cp(source, module_wasm_path);
            }
        }
    }

    /// Run wasm-strip on *.wasm files in wasm directory.
    fn optimize_wasm_files(&self) {
        log::info("Optimizing wasm files...");
        for contract in self.contracts() {
            // TODO: Optimize wasm files in modules
            command::wasm_strip(&contract.name, self.project.project_root());
        }
    }

    fn parse_contracts_names(&self) -> Vec<String> {
        match &self.contracts_names {
            Some(string) => remove_extra_spaces(string)
                .map(|string| {
                    string
                        .split(' ')
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                })
                .unwrap_or_else(|_| {
                    Error::FailedToParseArgument("contracts_names".to_string()).print_and_die()
                }),
            None => vec![],
        }
    }
}

fn remove_extra_spaces(input: &str) -> Result<String, &'static str> {
    // Ensure there are no other separators
    if input.chars().any(|c| c.is_whitespace() && c != ' ') {
        return Err("Input contains non-space whitespace characters");
    }

    let trimmed = input.split_whitespace().collect::<Vec<&str>>().join(" ");
    Ok(trimmed)
}
