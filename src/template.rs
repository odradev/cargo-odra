// TODO: Comments

/// Contract definition file template
const WASM_SOURCE_BUILDER: &str = r##"
fn main() {
    let contract_def = <#contract_fqn as odra::types::contract_def::HasContractDef>::contract_def();
    let code = odra::#backend_name::codegen::gen_contract(contract_def, "#contract_fqn".to_string());

    use std::fs::File;
    use std::io::prelude::*;
    let mut file = File::create("src/#contract_name_wasm.rs").unwrap();
    file.write_all(&code.to_string().into_bytes()).unwrap();
}
"##;

const MODULE_TEMPLATE: &str = r##"
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

pub fn wasm_source_builder(fqn: &str, contract_name: &str, backend_name: &str) -> String {
    WASM_SOURCE_BUILDER
        .replace("#contract_fqn", fqn)
        .replace("#contract_name", contract_name)
        .replace("#backend_name", backend_name)
}

pub fn module_template(module_name: &str) -> String {
    MODULE_TEMPLATE.replace("#module_name", module_name)
}

pub fn register_module_snippet(contract_name: &str, module_name: &str) -> String {
    MODULE_REGISTER
        .replace("#contract_name", contract_name)
        .replace("#module_name", module_name)
}
