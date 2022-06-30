use serde_derive::Deserialize;
use std::collections::HashMap;
use std::fs;

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct OdraConf {
    pub name: String,
    pub contracts: HashMap<String, Contract>,
}

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct Contract {
    pub path: String,
    pub name: String,
    pub fqn: String,
}

pub(crate) fn load_odra_conf() -> OdraConf {
    let odra_conf = fs::read_to_string("Odra.toml").unwrap();
    toml::from_str(odra_conf.as_str()).unwrap()
}

#[cfg(test)]
mod test {
    use crate::odra_toml::OdraConf;

    #[test]
    fn test_deserialize() {
        let toml_str = r#"
            name = "goralkocoin"
            [contracts]
            flipper = { path = "src/flipper.rs", name = "flipper", fqn = "Flipper::Flipper"}
            plascoin = { path = "src/plascoin.rs", name = "plascoin", fqn = "Plascoin::Plascoin" }
        "#;
        let decoded: OdraConf = toml::from_str(toml_str).unwrap();
        assert_eq!(decoded.contracts.get("plascoin").unwrap().name, "plascoin");
        assert_eq!(decoded.name, "goralkocoin");
    }
}
