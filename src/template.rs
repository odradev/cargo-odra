use ureq::get;

use crate::{
    consts::{MODULE_REGISTER, MODULE_TEMPLATE, WASM_SOURCE_BUILDER},
    errors::Error,
};

/// This module contains templates for generating new contracts.
pub struct TemplateGenerator {
    raw_repository_path: String,
    branch: String,
}

impl TemplateGenerator {
    pub fn new(repository_path: String, branch: String) -> Self {
        Self {
            raw_repository_path: repository_path,
            branch,
        }
    }

    fn template_path(&self, template_name: &str) -> String {
        format!(
            "{}/{}/templates/{}.template",
            self.raw_repository_path, self.branch, template_name
        )
    }

    fn fetch_template(&self, template_name: &str) -> String {
        let template_path = self.template_path(template_name);
        get(&template_path)
            .call()
            .unwrap_or_else(|_| Error::FailedToFetchTemplate(template_path.clone()).print_and_die())
            .into_string()
            .unwrap_or_else(|_| Error::FailedToParseTemplate(template_path.clone()).print_and_die())
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
