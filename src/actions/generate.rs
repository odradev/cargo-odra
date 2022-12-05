//! Module responsible for generating contracts code for user.

use std::path::PathBuf;

use convert_case::{Case, Casing};

use crate::{
    cargo_toml,
    command,
    errors::Error,
    log,
    odra_toml::{Contract, OdraToml},
    paths,
    template,
};

/// GenerateAction configuration.
pub struct GenerateAction {
    contract_name: String,
    module_ident: String,
}

/// GenerateAction implementation.
impl GenerateAction {
    /// Crate a new GenerateAction for a given contract.
    pub fn new(contract_name: String) -> GenerateAction {
        OdraToml::assert_exists();
        GenerateAction {
            contract_name: paths::to_snake_case(&contract_name),
            module_ident: contract_name.to_case(Case::UpperCamel),
        }
    }

    /// Main function that runs the generation action.
    pub fn generate_contract(&self) {
        log::info(format!("Adding new contract: {} ...", self.contract_name()));
        self.add_contract_file_to_src();
        self.update_lib_rs();
        self.update_odra_toml();
    }

    /// Returns the contract name.
    fn contract_name(&self) -> &str {
        &self.contract_name
    }

    /// Returns the module identifier. It is the struct name.
    fn module_ident(&self) -> &str {
        &self.module_ident
    }

    /// Returns project's crate name.
    fn project_crate_name(&self) -> String {
        paths::to_snake_case(cargo_toml::project_name())
    }

    /// Returns a path to file with contract definition.
    fn module_file_path(&self) -> PathBuf {
        paths::module_file_path(self.contract_name())
    }

    /// Crates a new module file in src directory.
    fn add_contract_file_to_src(&self) {
        // Rename module name.
        let contract_body = template::module_template(self.module_ident());

        // Make sure the file do not exists.
        let path = self.module_file_path();
        if path.exists() {
            Error::FileAlreadyExists(path).print_and_die();
        }

        // Write to file.
        command::write_to_file(path, &contract_body);
    }

    /// Append `mod` section to lib.rs.
    fn update_lib_rs(&self) {
        // Prepare code to add.
        let register_module_code =
            template::register_module_snippet(self.contract_name(), self.module_ident());

        // Write to file.
        command::append_file(paths::project_lib_rs(), &register_module_code);

        // Print info.
        log::info(format!("Added to src/lib.rs:\n{register_module_code}"));
    }

    /// Add contract definition to Odra.toml.
    fn update_odra_toml(&self) {
        let mut odra_toml = OdraToml::load();
        let contract_name = self.contract_name();

        // Check if Odra.toml has already a contract.
        let exists = odra_toml.has_contract(contract_name);
        if exists {
            Error::ContractAlreadyInOdraToml(String::from(contract_name)).print_and_die();
        }

        // Add contract to Odra.toml.
        odra_toml.contracts.push(Contract {
            name: self.contract_name().to_string(),
            fqn: format!(
                "{}::{}::{}",
                self.project_crate_name(),
                self.contract_name(),
                self.module_ident()
            ),
        });

        // Write to file.
        odra_toml.save();

        // Print info.
        log::info("Added contract to Odra.toml.");
    }
}
