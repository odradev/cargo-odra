//! Constants used by cargo odra

/// Name of the Odra framework crate
pub const ODRA_CRATE_NAME: &str = "odra";

/// Name of the file which holds Odra configuration
pub const ODRA_TOML_FILENAME: &str = "Odra.toml";

/// Casper backend name.
pub const ODRA_CASPER_BACKEND: &str = "casper";

/// Contract definition file template
pub const DEF_RS: &str = r##"
fn main() {
    let contract_def = <#contract_fqn as odra::types::contract_def::HasContractDef>::contract_def();
    let code = odra::#backend_name::codegen::gen_contract(contract_def, "#contract_fqn".to_string());

    use std::fs::File;
    use std::io::prelude::*;
    let mut file = File::create("src/#contract_name_wasm.rs").unwrap();
    file.write_all(&code.to_string().into_bytes()).unwrap();
}
"##;

/// Template repository path.
pub const ODRA_TEMPLATE_GH_REPO: &str = "odradev/odra-template";

pub const MODULE_NAME_SYMBOL: &str = "#module_name";

pub const CONTRACT_NAME_SYMBOL: &str = "#contract_name";

pub const MODULE_TEMPLATE: &str = r##"
use odra::Variable;

/// A #module_name module storage definition.
#[odra::module]
pub struct #module_name {
    value: Variable<bool>,
}

/// Module entrypoints implementation.
#[odra::module]
impl #module_name {
    /// #module_name constructor.
    /// Initializes the contract with the value of value.
    #[odra(init)]
    pub fn initial_settings(&self) {
        self.value.set(false);
    }

    /// Replaces the current value with the passed argument.
    pub fn set(&self, value: bool) {
        self.value.set(value);
    }

    /// Retrieves value from the storage. 
    /// If the value has never been set, the default value is returned.
    pub fn get(&self) -> bool {
        self.value.get_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::#module_name;

    #[test]
    fn it_works() {
        let contract = #module_name::deploy_initial_settings();
        assert!(!contract.get());
        contract.set(true);
        assert!(contract.get());
    }
}
"##;

pub const MODULE_REGISTER: &str = r##"pub mod #contract_name;
pub use #contract_name::{#module_name, #module_nameRef};
"##;
