//! Module for generating contracts schema.

use super::utils;
use crate::{command, errors::Error, log, project::Project};

/// SchemaAction configuration.
pub struct SchemaAction<'a> {
    project: &'a Project,
    contracts_names: Option<String>,
}

impl<'a> SchemaAction<'a> {
    /// Crate a new SchemaAction for a given configuration.
    pub fn new(project: &'a Project, contracts_names: Option<String>) -> Self {
        SchemaAction {
            project,
            contracts_names,
        }
    }
}

impl SchemaAction<'_> {
    /// Main function that runs the whole workflow.
    pub fn build(&self) {
        utils::check_target_requirements();
        utils::validate_contract_name_argument(self.project, self.contracts_names());
        self.generate_schema_files();
    }

    /// Generates *_schema.json files.
    fn generate_schema_files(&self) {
        log::info("Generating schema files...");
        let contracts =
            utils::contracts(self.project, self.contracts_names()).unwrap_or_else(|_| {
                Error::FailedToParseArgument("contracts_names".to_string()).print_and_die()
            });
        for contract in contracts {
            command::cargo_generate_schema_files(
                self.project.project_root(),
                &contract.struct_name(),
                &contract.crate_name(self.project),
            );
        }
    }

    fn contracts_names(&self) -> String {
        self.contracts_names.clone().unwrap_or_default()
    }
}
