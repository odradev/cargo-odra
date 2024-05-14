//! Module managing Odra.toml configuration.

use std::path::{Path, PathBuf};

use serde_derive::{Deserialize, Serialize};

use crate::{
    command,
    errors::{Error, Error::MalformedFqn},
    project::Project,
};

/// Struct describing contract.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Contract {
    pub fqn: String,
}

impl Contract {
    pub fn module_name(&self) -> String {
        self.fqn
            .split_terminator("::")
            .next()
            .unwrap_or_else(|| MalformedFqn.print_and_die())
            .replace("-", "_")
            .to_string()
    }

    pub fn module_crate_name(&self, project: &Project) -> String {
        if project.is_workspace() {
            project
                .members
                .iter()
                .find(|m| m.name.replace("-", "_") == self.module_name())
                .unwrap_or_else(|| {
                    Error::CrateOfContractNotFound(self.module_name()).print_and_die()
                })
                .name
                .clone()
        } else {
            project.project_crate_name()
        }
    }

    pub fn crate_name(&self, project: &Project) -> String {
        if project.is_workspace() {
            self.module_crate_name(project)
        } else {
            project.project_crate_name()
        }
    }

    pub fn struct_name(&self) -> String {
        self.fqn
            .split_terminator("::")
            .last()
            .unwrap_or_else(|| MalformedFqn.print_and_die())
            .to_string()
    }
}

/// Odra configuration.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct OdraToml {
    /// Contracts in the project.
    pub contracts: Vec<Contract>,
    #[serde(skip)]
    pub location: PathBuf,
}

impl OdraToml {
    /// Loads configuration from Odra.toml file.
    pub fn load(location: &Path) -> OdraToml {
        let odra_conf = command::read_file_content(location.to_path_buf());
        let mut odra_toml: OdraToml = match odra_conf {
            Ok(conf_file) => toml::from_str(conf_file.as_str()).unwrap(),
            Err(_) => Error::OdraTomlNotFound(location.to_path_buf()).print_and_die(),
        };

        odra_toml.location = location.to_path_buf();
        odra_toml
    }

    /// Saves configuration into Odra.toml file.
    pub fn save(&self) {
        let content = toml::to_string(&self).unwrap();
        command::write_to_file(self.location.clone(), &content);
    }

    /// Check if the contract is defined in Odra.toml file.
    pub fn has_contract(&self, contract_name: &str) -> bool {
        self.contracts
            .iter()
            .any(|c| c.struct_name() == contract_name)
    }

    /// Check if any contract in Odra.toml is a part of a crate with given name
    pub fn crate_has_contracts(&self, crate_name: &str) -> bool {
        self.contracts.iter().any(|c| {
            c.fqn
                .split_terminator("::")
                .next()
                .unwrap_or_else(|| Error::MalformedFqn.print_and_die())
                == crate_name.replace("-", "_")
        })
    }
}
