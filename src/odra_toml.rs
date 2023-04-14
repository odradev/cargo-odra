//! Module managing Odra.toml configuration.

use serde_derive::{Deserialize, Serialize};

use crate::project::Project;
use crate::{command, errors::Error, paths};

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
}

impl OdraToml {
    /// Loads configuration from Odra.toml file.
    pub fn load() -> Option<OdraToml> {
        let odra_conf_location = Project::find_odra_toml(None);
        match odra_conf_location {
            None => None,
            Some(location) => {
                let odra_conf = command::read_file_content(location.clone());
                match odra_conf {
                    Ok(conf_file) => {
                        println!("found odra toml: {}", location.to_str().unwrap());
                        Some(toml::from_str(conf_file.as_str()).unwrap())
                    }
                    Err(_) => None,
                }
            }
        }
    }

    /// Saves configuration into Odra.toml file.
    pub fn save(&self) {
        let content = toml::to_string(&self).unwrap();
        command::write_to_file(paths::odra_toml(), &content);
    }

    /// Check if the contract is defined in Odra.toml file.
    pub fn has_contract(&self, contract_name: &str) -> bool {
        self.contracts.iter().any(|c| c.name == contract_name)
    }
}
