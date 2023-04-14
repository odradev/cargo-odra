/// Wasm source builder file template.
const WASM_SOURCE_BUILDER: &str = r##"
fn main() {
    let ident = <#contract_fqn as odra::types::contract_def::HasIdent>::ident();
    let entrypoints = <#contract_fqn as odra::types::contract_def::HasEntrypoints>::entrypoints();
    let events = <#contract_fqn as odra::types::contract_def::HasEvents>::events();
    let code = odra::#backend_name::codegen::gen_contract(
        ident,
        entrypoints,
        events,
        "#contract_fqn".to_string()
    );

    use std::fs::File;
    use std::io::prelude::*;
    let mut file = File::create("src/#contract_name_wasm.rs").unwrap();
    file.write_all(&code.to_string().into_bytes()).unwrap();
}
"##;

/// Module file template.
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
    pub fn initial_settings(&mut self) {
        self.value.set(false);
    }

    /// Replaces the current value with the passed argument.
    pub fn set(&mut self, value: bool) {
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
    use super::#module_nameDeployer;

    #[test]
    fn it_works() {
        let mut contract = #module_nameDeployer::initial_settings();
        assert!(!contract.get());
        contract.set(true);
        assert!(contract.get());
    }
}
"##;

/// Code for src/lib.rs that registers new module.
pub const MODULE_REGISTER: &str = r##"
pub mod #contract_name;
pub use #contract_name::{#module_name, #module_nameRef};
"##;

/// Returns content of the new _builder.rs file.
pub fn wasm_source_builder(fqn: &str, contract_name: &str, backend_name: &str) -> String {
    WASM_SOURCE_BUILDER
        .replace("#contract_fqn", fqn)
        .replace("#contract_name", contract_name)
        .replace("#backend_name", backend_name)
}

/// Returns content of the new module file.
pub fn module_template(module_name: &str) -> String {
    MODULE_TEMPLATE.replace("#module_name", module_name)
}

/// Returns code for src/lib.rs that registers a new module.
pub fn register_module_snippet(contract_name: &str, module_name: &str) -> String {
    MODULE_REGISTER
        .replace("#contract_name", contract_name)
        .replace("#module_name", module_name)
}
