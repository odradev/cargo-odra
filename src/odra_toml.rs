use crate::Backend;

use crate::errors::Error;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

const ODRA_TOML_FILENAME: &str = "Odra.toml";

#[derive(Deserialize, Serialize, Debug, Clone)]
pub(crate) struct OdraConf {
    pub name: String,
    pub contracts: HashMap<String, Contract>,
    pub backends: Option<HashMap<String, Backend>>,
}

impl OdraConf {
    pub fn load() -> OdraConf {
        let odra_conf = fs::read_to_string(ODRA_TOML_FILENAME);
        match odra_conf {
            Ok(conf_file) => toml::from_str(conf_file.as_str()).unwrap(),
            Err(_) => {
                Error::NotAnOdraProject.print_and_die();
            }
        }
    }

    pub fn save(&self) {
        let content = toml::to_string(&self).unwrap();
        fs::write(ODRA_TOML_FILENAME, content).unwrap();
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub(crate) struct Contract {
    pub path: String,
    pub name: String,
    pub fqn: String,
}

pub fn assert_odra_toml() {
    if !Path::new(ODRA_TOML_FILENAME).exists() {
        Error::NotAnOdraProject.print_and_die();
    }
}
