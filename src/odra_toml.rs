//! Module managing Odra.toml configuration.

use std::path::PathBuf;

use serde_derive::{Deserialize, Serialize};

use crate::{command, errors::Error};

/// Struct describing contract.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Contract {
    /// Name of the contract
    pub name: String,
    /// Fully Qualified Name of the contract struct
    pub fqn: String,
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
    pub fn load(location: PathBuf) -> OdraToml {
        let odra_conf = command::read_file_content(location.clone());
        let mut odra_toml: OdraToml = match odra_conf {
            Ok(conf_file) => toml::from_str(conf_file.as_str()).unwrap(),
            Err(_) => Error::OdraTomlNotFound(location).print_and_die(),
        };

        odra_toml.location = location;
        odra_toml
    }

    /// Saves configuration into Odra.toml file.
    pub fn save(&self) {
        let content = toml::to_string(&self).unwrap();
        command::write_to_file(self.location.clone(), &content);
    }

    /// Check if the contract is defined in Odra.toml file.
    pub fn has_contract(&self, contract_name: &str) -> bool {
        self.contracts.iter().any(|c| c.name == contract_name)
    }
}
