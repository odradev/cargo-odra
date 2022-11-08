//! Module responsible for generating contracts code for user
use crate::errors::Error;
use crate::odra_toml::{Contract, OdraToml};
use crate::{cli::GenerateCommand, log, template};

use convert_case::{Case, Casing};
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

/// GenerateAction struct
pub struct GenerateAction {
    contract_name: String,
    module_ident: String,
}

impl GenerateAction {
    pub fn new(generate: GenerateCommand) -> GenerateAction {
        OdraToml::assert_exists();
        GenerateAction {
            contract_name: generate.contract_name.to_case(Case::Snake),
            module_ident: generate.contract_name.to_case(Case::UpperCamel),
        }
    }

    pub fn generate_contract(&self) {
        log::info(format!("Adding new contract: {} ...", self.contract_name()));
        self.add_contract_file_to_src();
        self.update_lib_rs();
        self.update_odra_toml();
    }

    fn contract_name(&self) -> &str {
        &self.contract_name
    }

    fn module_ident(&self) -> &str {
        &self.module_ident
    }

    fn module_file_path(&self) -> PathBuf {
        PathBuf::from("src")
            .join(self.contract_name())
            .with_extension("rs")
    }

    fn add_contract_file_to_src(&self) {
        // Rename module name.
        let contract_body = template::module_template(self.module_ident());

        // Make sure the file do not exists.
        let path = self.module_file_path();
        if path.exists() {
            Error::FileAlreadyExists(path).print_and_die();
        }

        // Write to file.
        fs::write(path, contract_body).unwrap();
    }

    fn update_lib_rs(&self) {
        // Read src/lib.rs.
        let mut lib_rs = OpenOptions::new()
            .write(true)
            .append(true)
            .open("src/lib.rs")
            .unwrap();

        // Prepare code to add.
        let register_module_code =
            template::register_module_snippet(self.contract_name(), self.module_ident());

        // Write to file.
        writeln!(lib_rs, "{}", &register_module_code).unwrap();
        lib_rs.flush().unwrap();

        // Print info.
        log::info(format!("Added to src/lib.rs: \n\n{}", register_module_code));
    }

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
            // @TODO: Prepend with cargo module name.
            fqn: format!("{}::{}", self.contract_name(), self.module_ident()),
        });

        // Write to file.
        odra_toml.save();

        // Print info.
        log::info("Added contract to Odra.toml.");
    }
}
