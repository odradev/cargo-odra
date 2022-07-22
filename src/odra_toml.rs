use crate::Backend;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::{fs, process};

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
                panic!("Odra.toml file is missing. Is Odra initialized?")
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
        println!("This command can be executed only in folder with Odra project.");
        process::exit(1);
    }
}

#[cfg(test)]
mod test {
    use crate::odra_toml::OdraConf;

    #[test]
    fn test_deserialize() {
        let toml_str = r#"
            name = "goralkocoin"
            [contracts]
            flipper = { path = "src/flipper.rs", name = "flipper", fqn = "Flipper::Flipper" }
            plascoin = { path = "src/plascoin.rs", name = "plascoin", fqn = "Plascoin::Plascoin" }
            [backends]
            casper = { path = "../odra-casper", name = "casper" }
            casper2 = { path = "https://github.com/odradev/odra-casper", branch = "develop", name = "casper" }
        "#;
        let decoded: OdraConf = toml::from_str(toml_str).unwrap();
        assert_eq!(decoded.contracts.get("plascoin").unwrap().name, "plascoin");
        assert_eq!(decoded.name, "goralkocoin");
        assert_eq!(
            decoded
                .backends
                .clone()
                .unwrap()
                .get("casper")
                .unwrap()
                .path,
            "../odra-casper".to_string()
        );
        assert_eq!(
            decoded
                .backends
                .clone()
                .unwrap()
                .get("casper")
                .unwrap()
                .branch,
            None
        );
        assert_eq!(
            decoded
                .backends
                .clone()
                .unwrap()
                .get("casper")
                .unwrap()
                .name,
            "casper".to_string()
        );
        assert_eq!(
            decoded.backends.unwrap().get("casper2").unwrap().branch,
            Some("develop".to_string())
        );
    }
}
