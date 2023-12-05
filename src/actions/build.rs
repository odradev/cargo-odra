//! Module for managing and building backends.

use std::path::Path;

use cargo_toml::{Dependency, DependencyDetail, DepsSet};

use crate::{
    cargo_toml::odra_raw_dependency,
    command,
    consts::ODRA_TEMPLATE_GH_RAW_REPO,
    errors::Error,
    log,
    odra_toml::Contract,
    paths::{self, BuilderPaths},
    project::Project,
    template::TemplateGenerator,
};

/// BuildAction configuration.
pub struct BuildAction<'a> {
    backend: String,
    contracts_names: Option<String>,
    builder_paths: BuilderPaths,
    project: &'a Project,
    template_generator: TemplateGenerator,
}

/// BuildAction implementation.
impl<'a> BuildAction<'a> {
    /// Crate a new BuildAction for a given backend.
    pub fn new(project: &'a Project, backend: String, contracts_names: Option<String>) -> Self {
        BuildAction {
            backend: backend.clone(),
            contracts_names,
            builder_paths: BuilderPaths::new(backend, project.project_root.clone()),
            project,
            template_generator: TemplateGenerator::new(
                ODRA_TEMPLATE_GH_RAW_REPO.to_string(),
                project.project_odra_location(),
            ),
        }
    }
}

impl BuildAction<'_> {
    /// Returns the name of the backend.
    /// It is also the name of the Odra's feature.
    pub fn backend_name(&self) -> String {
        self.backend.clone()
    }

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
        for contract in self.contracts() {
            command::cargo_build_wasm_files(self.project.project_root(), &contract.name);
            let source = paths::wasm_path_in_target("contract", self.project.project_root());
            let target = paths::wasm_path_in_wasm_dir(
                &paths::to_snake_titlecase(&contract.name),
                self.project.project_root(),
            );
            log::info(format!("Saving {}", target.display()));
            command::cp(source, target);
        }
    }

    /// Run wasm-strip on *.wasm files in wasm directory.
    fn optimize_wasm_files(&self) {
        log::info("Optimizing wasm files...");
        for contract in self.contracts() {
            command::wasm_strip(
                &paths::to_snake_titlecase(contract.name),
                self.project.project_root(),
            );
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
