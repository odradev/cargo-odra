//! Module responsible for generating contracts code for user.

use std::path::PathBuf;

use crate::{
    command,
    consts::ODRA_TEMPLATE_GH_RAW_REPO,
    errors::Error,
    log,
    odra_toml::Contract,
    paths::{to_camel_case, to_snake_case},
    project::Project,
    template::TemplateGenerator,
};

/// GenerateAction configuration.
pub struct GenerateAction<'a> {
    project: &'a Project,
    contract_name: String,
    contract_module_ident: String,
    module_root: PathBuf,
    module_name: Option<String>,
    template_generator: TemplateGenerator,
}

/// GenerateAction implementation.
impl<'a> GenerateAction<'a> {
    /// Crate a new GenerateAction for a given contract.
    pub fn new(project: &'a Project, contract_name: String, module_name: Option<String>) -> Self {
        if project.is_workspace() && module_name.is_none() {
            Error::ModuleNotProvided.print_and_die();
        }

        GenerateAction {
            project,
            contract_name: contract_name.clone(),
            contract_module_ident: to_snake_case(contract_name),
            module_root: project.module_root(module_name.clone()),
            module_name,
            template_generator: TemplateGenerator::new(
                ODRA_TEMPLATE_GH_RAW_REPO.to_string(),
                project.project_odra_location(),
            ),
        }
    }
}

impl GenerateAction<'_> {
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

    /// Returns the contract identifier. It is the struct name.
    fn contract_struct_name(&self) -> String {
        let contract_name = self.contract_name();
        to_camel_case(contract_name)
    }

    /// Returns the module name.
    fn module_name(&self) -> String {
        to_snake_case(self.contract_name())
    }

    /// Returns the module Ref identifier.
    fn module_ref_ident(&self) -> String {
        format!("{}Ref", self.contract_module_ident)
    }

    /// Returns a path to file with contract definition.
    fn module_file_path(&self) -> PathBuf {
        self.module_root
            .join("src")
            .join(self.module_name())
            .with_extension("rs")
    }

    /// Crates a new module file in src directory.
    fn add_contract_file_to_src(&self) {
        // Rename module name.
        let contract_body = self
            .template_generator
            .module_template(&self.contract_struct_name())
            .unwrap_or_else(|err| err.print_and_die());

        // Make sure the file do not exist.
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
        let register_module_code = self
            .template_generator
            .register_module_snippet(&self.module_name(), &self.contract_struct_name())
            .unwrap_or_else(|err| err.print_and_die());

        // Read the file.
        let lib_rs_path = self.module_root.join("src/lib.rs");
        let lib_rs = command::read_file_content(lib_rs_path)
            .unwrap_or_else(|_| Error::LibRsNotFound.print_and_die());

        // If the file already has module registered, throw an error.
        if lib_rs.contains(&register_module_code) {
            Error::ModuleAlreadyInLibRs(String::from(self.contract_name())).print_and_die();
        }

        // Check if he file might have the module registered in another form.
        if lib_rs.contains(self.contract_name())
            || (lib_rs.contains(&self.contract_struct_name())
                && lib_rs.contains(&self.module_ref_ident()))
        {
            log::warn(format!(
                "src/lib.rs probably already has {} enabled. Skipping.",
                self.contract_name()
            ));
            return;
        }

        // Write to file.
        command::append_file(self.module_root.join("src/lib.rs"), &register_module_code);

        // Print info.
        log::info(format!("Added to src/lib.rs:\n{register_module_code}"));
    }

    /// Add contract definition to Odra.toml.
    fn update_odra_toml(&self) {
        let mut odra_toml = self.project.odra_toml();
        let contract_name = self.contract_struct_name();

        // Check if Odra.toml has already a contract.
        let exists = odra_toml.has_contract(contract_name.as_str());
        if exists {
            Error::ContractAlreadyInOdraToml(contract_name).print_and_die();
        }

        let fqn = match self.module_name.clone() {
            None => {
                format!("{}::{}", self.module_name(), contract_name)
            }
            Some(module_name) => {
                format!(
                    "{}::{}::{}",
                    self.project.crate_name(Some(module_name)),
                    self.contract_module_ident,
                    contract_name
                )
            }
        };

        // Add contract to Odra.toml.
        odra_toml.contracts.push(Contract { fqn });

        // Write to file.
        odra_toml.save();

        // Print info.
        log::info("Added contract to Odra.toml.");
    }
}
