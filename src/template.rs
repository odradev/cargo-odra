use serde_derive::{Deserialize, Serialize};
use ureq::{get, serde_json};

use crate::{
    command::read_file_content,
    consts::{MODULE_REGISTER, MODULE_TEMPLATE, TEMPLATES_JSON_PATH},
    errors::Error,
    project::OdraLocation,
};

/// Template type.
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum TemplateType {
    Contract,
    Project,
    Internal,
}

/// Struct representing Template.
#[derive(Serialize, Deserialize, Debug)]
pub struct Template {
    pub name: String,
    pub description: String,
    pub path: String,
    pub template_type: TemplateType,
}

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

    fn template_path(&self, template_path: &str, branch: String) -> String {
        format!("{}/{}/{}", self.raw_repository_path, branch, template_path)
    }

    /// Fetches templates.json
    pub fn fetch_templates(&self) -> Vec<Template> {
        match self.odra_location.clone() {
            OdraLocation::Local(path) => {
                let path = path.join(TEMPLATES_JSON_PATH);
                let path_string = path.to_str().unwrap().to_string();
                serde_json::from_str(&read_file_content(path).unwrap_or_else(|_| {
                    Error::FailedToFetchTemplatesFile(path_string.clone()).print_and_die()
                }))
                .unwrap_or_else(|_| Error::FailedToParseTemplatesFile(path_string).print_and_die())
            }
            OdraLocation::Remote(_, branch) => {
                let branch = branch.unwrap_or_else(|| "releases/latest".to_string());
                let template_path = self.template_path(TEMPLATES_JSON_PATH, branch);
                let templates_json = Self::download_template(&template_path);
                serde_json::from_str(&templates_json).unwrap_or_else(|_| {
                    Error::FailedToParseTemplatesFile(template_path).print_and_die()
                })
            }
            OdraLocation::CratesIO(version) => {
                let branch = format!("release/{version}");
                let template_path = self.template_path(TEMPLATES_JSON_PATH, branch);
                let templates_json = Self::download_template(&template_path);
                serde_json::from_str(&templates_json).unwrap_or_else(|_| {
                    Error::FailedToParseTemplatesFile(template_path).print_and_die()
                })
            }
        }
    }

    pub fn find_template(&self, template_name: &str) -> Template {
        self.fetch_templates()
            .into_iter()
            .find(|template| template.name == template_name)
            .unwrap_or_else(|| Error::TemplateNotFound(template_name.to_owned()).print_and_die())
    }

    pub fn fetch_template(&self, template_name: &str) -> String {
        let template = self.find_template(template_name);

        if template.template_type == TemplateType::Project {
            Error::IncorrectTemplateType.print_and_die()
        }

        match self.odra_location.clone() {
            OdraLocation::Local(path) => {
                let path = path.join(template.path);
                read_file_content(path).unwrap_or_else(|_| {
                    Error::FailedToFetchTemplate(template_name.to_owned()).print_and_die()
                })
            }
            OdraLocation::Remote(_, branch) => {
                let branch = branch.unwrap_or_else(|| "releases/latest".to_string());
                let template_path = self.template_path(&template.path, branch);
                Self::download_template(&template_path)
            }
            OdraLocation::CratesIO(version) => {
                let branch = format!("release/{version}");
                let template_path = self.template_path(&template.path, branch);
                Self::download_template(&template_path)
            }
        }
    }

    fn download_template(template_path: &String) -> String {
        get(&template_path.clone())
            .call()
            .unwrap_or_else(|_| {
                Error::FailedToFetchTemplate(template_path.to_string()).print_and_die()
            })
            .into_string()
            .unwrap_or_else(|_| {
                Error::FailedToParseTemplate(template_path.to_string()).print_and_die()
            })
    }

    /// Returns content of the new module file.
    pub fn module_template(
        &self,
        module_name: &str,
        template_name: Option<String>,
    ) -> Result<String, Error> {
        let template_name = template_name.unwrap_or_else(|| MODULE_TEMPLATE.to_string());
        Ok(self
            .fetch_template(&template_name)
            .replace("#module_name", module_name))
    }

    /// Returns code for src/lib.rs that registers a new module.
    pub fn register_module_snippet(
        &self,
        contract_name: &str,
        module_name: &str,
    ) -> Result<String, Error> {
        Ok(self
            .fetch_template(MODULE_REGISTER)
            .replace("#contract_name", contract_name)
            .replace("#module_name", module_name))
    }
}
