//! Module managing Odra.toml configuration

use crate::consts::ODRA_TOML_FILENAME;
use crate::errors::Error;
use serde_derive::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Odra configuration
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct OdraToml {
    /// Contracts in the project.
    pub contracts: Vec<Contract>,
}

impl OdraToml {
    /// Loads configuration from Odra.toml file.
    pub fn load() -> OdraToml {
        let odra_conf = fs::read_to_string(ODRA_TOML_FILENAME);
        match odra_conf {
            Ok(conf_file) => toml::from_str(conf_file.as_str()).unwrap(),
            Err(_) => {
                Error::NotAnOdraProject.print_and_die();
            }
        }
    }

    /// Exits program if there is no Odra.toml file.
    pub fn assert_exists() {
        if !Path::new(ODRA_TOML_FILENAME).exists() {
            Error::NotAnOdraProject.print_and_die();
        }
    }

    /// Saves configuration into Odra.toml file.
    pub fn save(&self) {
        let content = toml::to_string(&self).unwrap();
        fs::write(ODRA_TOML_FILENAME, content).unwrap();
    }

    /// Check if the contract is defined in Odra.toml file.
    pub fn has_contract(&self, contract_name: &str) -> bool {
        self.contracts.iter().any(|c| c.name == contract_name)
    }
}

/// Struct describing contract
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Contract {
    /// Name of the contract
    pub name: String,
    /// Fully Qualified Name of the contract struct
    pub fqn: String,
}
