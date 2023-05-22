use ureq::get;

use crate::{
    command::read_file_content,
    consts::{MODULE_REGISTER, MODULE_TEMPLATE, WASM_SOURCE_BUILDER},
    errors::Error,
    project::OdraLocation,
};

/// This module contains templates for generating new contracts.
pub struct TemplateGenerator {
    raw_repository_path: String,
    odra_location: OdraLocation,
}

impl TemplateGenerator {
    pub fn new(repository_path: String, odra_location: OdraLocation) -> Self {
        Self {
            raw_repository_path: repository_path,
            odra_location,
        }
    }

    fn template_path(&self, template_name: &str, branch: String) -> String {
        format!(
            "{}/{}/templates/{}.rs.template",
            self.raw_repository_path, branch, template_name
        )
    }

    fn fetch_template(&self, template_name: &str) -> String {
        match self.odra_location.clone() {
            OdraLocation::Local(path) => {
                let path = path
                    .join("templates")
                    .join(template_name)
                    .with_extension("rs.template");
                read_file_content(path).unwrap()
            }
            OdraLocation::Remote(_, branch) => {
                let branch = branch.unwrap_or_else(|| "releases/latest".to_string());
                let template_path = self.template_path(template_name, branch);
                get(&template_path)
                    .call()
                    .unwrap_or_else(|_| {
                        Error::FailedToFetchTemplate(template_path.clone()).print_and_die()
                    })
                    .into_string()
                    .unwrap_or_else(|_| {
                        Error::FailedToParseTemplate(template_path.clone()).print_and_die()
                    })
            }
            OdraLocation::CratesIO(version) => {
                let branch = format!("release/{}", version);
                let template_path = self.template_path(template_name, branch);
                get(&template_path)
                    .call()
                    .unwrap_or_else(|_| {
                        Error::FailedToFetchTemplate(template_path.clone()).print_and_die()
                    })
                    .into_string()
                    .unwrap_or_else(|_| {
                        Error::FailedToParseTemplate(template_path.clone()).print_and_die()
                    })
            }
        }
    }

    /// Returns content of the new _builder.rs file.
    pub fn wasm_source_builder(
        &self,
        fqn: &str,
        contract_name: &str,
        backend_name: &str,
    ) -> String {
        self.fetch_template(WASM_SOURCE_BUILDER)
            .replace("#contract_fqn", fqn)
            .replace("#contract_name", contract_name)
            .replace("#backend_name", backend_name)
    }

    /// Returns content of the new module file.
    pub fn module_template(&self, module_name: &str) -> String {
        self.fetch_template(MODULE_TEMPLATE)
            .replace("#module_name", module_name)
    }

    /// Returns code for src/lib.rs that registers a new module.
    pub fn register_module_snippet(&self, contract_name: &str, module_name: &str) -> String {
        self.fetch_template(MODULE_REGISTER)
            .replace("#contract_name", contract_name)
            .replace("#module_name", module_name)
    }
}
