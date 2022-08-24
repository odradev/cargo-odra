//! Module managing Odra.toml configuration
use crate::Backend;

use crate::consts::ODRA_TOML_FILENAME;
use crate::errors::Error;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Odra configuration
#[derive(Deserialize, Serialize, Debug, Clone)]
pub(crate) struct OdraConf {
    /// Project name
    pub name: String,
    /// Contracts in the project
    pub contracts: HashMap<String, Contract>,
    /// Backends attached to the project
    pub backends: Option<HashMap<String, Backend>>,
}

impl OdraConf {
    /// Loads configuration from Odra.toml file
    pub fn load() -> OdraConf {
        let odra_conf = fs::read_to_string(ODRA_TOML_FILENAME);
        match odra_conf {
            Ok(conf_file) => toml::from_str(conf_file.as_str()).unwrap(),
            Err(_) => {
                Error::NotAnOdraProject.print_and_die();
            }
        }
    }

    /// Saves configuration into Odra.toml file
    pub fn save(&self) {
        let content = toml::to_string(&self).unwrap();
        fs::write(ODRA_TOML_FILENAME, content).unwrap();
    }
}

/// Struct describing contract
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Contract {
    /// Path to the contract file
    pub path: String,
    /// Name of the contract
    pub name: String,
    /// Fully Qualified Name of the contract struct
    pub fqn: String,
}

/// Exits program if there is no Odra.toml file
pub fn assert_odra_toml() {
    if !Path::new(ODRA_TOML_FILENAME).exists() {
        Error::NotAnOdraProject.print_and_die();
    }
}
