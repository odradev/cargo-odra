use ureq::get;

use crate::{
    command::read_file_content,
    consts::{MODULE_REGISTER, MODULE_TEMPLATE},
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

    fn fetch_template(&self, template_name: &str) -> Result<String, Error> {
        match self.odra_location.clone() {
            OdraLocation::Local(path) => {
                let path = path
                    .join("templates")
                    .join(template_name)
                    .with_extension("rs.template");
                read_file_content(path)
                    .map_err(|_| Error::FailedToFetchTemplate(template_name.to_owned()))
            }
            OdraLocation::Remote(_, branch) => {
                let branch = branch.unwrap_or_else(|| "releases/latest".to_string());
                let template_path = self.template_path(template_name, branch);
                get(&template_path)
                    .call()
                    .map_err(|_| Error::FailedToFetchTemplate(template_path.clone()))
                    .and_then(|res| {
                        res.into_string()
                            .map_err(|_| Error::FailedToParseTemplate(template_path.clone()))
                    })
            }
            OdraLocation::CratesIO(version) => {
                let branch = format!("release/{}", version);
                let template_path = self.template_path(template_name, branch);
                get(&template_path)
                    .call()
                    .map_err(|_| Error::FailedToFetchTemplate(template_path.clone()))
                    .and_then(|res| {
                        res.into_string()
                            .map_err(|_| Error::FailedToParseTemplate(template_path.clone()))
                    })
            }
        }
    }

    /// Returns content of the new module file.
    pub fn module_template(&self, module_name: &str) -> Result<String, Error> {
        Ok(self
            .fetch_template(MODULE_TEMPLATE)?
            .replace("#module_name", module_name))
    }

    /// Returns code for src/lib.rs that registers a new module.
    pub fn register_module_snippet(
        &self,
        contract_name: &str,
        module_name: &str,
    ) -> Result<String, Error> {
        Ok(self
            .fetch_template(MODULE_REGISTER)?
            .replace("#contract_name", contract_name)
            .replace("#module_name", module_name))
    }
}
