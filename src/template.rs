use cargo_generate::{GenerateArgs, TemplatePath, Vcs};
use chrono::Utc;
use ureq::get;

use crate::{
    command::read_file_content,
    consts::{
        GEN_CONTRACT_MOD,
        MATCH_CONTRACT_NAME,
        MODULE_REGISTER,
        MODULE_TEMPLATE,
        ODRA_TEMPLATE_GH_REPO,
        WASM_SINGLE_SOURCE_BUILDER,
    },
    errors::Error,
    paths,
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

    /// Returns content of the new contracts_builder.rs file.
    pub fn wasm_source_builder(
        &self,
        contracts_names: Vec<(String, String)>,
        backend_name: &str,
    ) -> Result<String, Error> {
        let contract_matcher = contracts_names
            .iter()
            .map(|(_, contract_name)| {
                self.fetch_template(MATCH_CONTRACT_NAME)
                    .map(|t| t.replace("#contract_name", contract_name))
            })
            .collect::<Result<Vec<_>, Error>>()?
            .join("\n");

        let gen_contract_mod = contracts_names
            .iter()
            .map(|(fqn, contract_name)| {
                self.fetch_template(GEN_CONTRACT_MOD).map(|t| {
                    t.replace("#fqn", fqn)
                        .replace("#contract_name", contract_name)
                        .replace("#backend_name", backend_name)
                })
            })
            .collect::<Result<Vec<_>, Error>>()?
            .join("\n");

        Ok(self
            .fetch_template(WASM_SINGLE_SOURCE_BUILDER)?
            .replace("#code_gen_modules", &gen_contract_mod)
            .replace("#contract_matcher", &contract_matcher))
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

    pub fn generate_from_template(path: &str, template_path: TemplatePath, init: bool) {
        cargo_generate::generate(GenerateArgs {
            template_path,
            list_favorites: false,
            name: Some(paths::to_snake_case(path)),
            force: true,
            verbose: false,
            template_values_file: None,
            silent: false,
            config: None,
            vcs: Some(Vcs::Git),
            lib: false,
            bin: false,
            ssh_identity: None,
            define: vec![format!("date={}", Utc::now().format("%Y-%m-%d"))],
            init,
            destination: None,
            force_git_init: false,
            allow_commands: false,
            overwrite: false,
            other_args: None,
        })
        .unwrap_or_else(|e| {
            Error::FailedToGenerateProjectFromTemplate(e.to_string()).print_and_die();
        });
    }

    pub fn odra_template_path(template: &str, odra_location: &OdraLocation) -> TemplatePath {
        let template_path = match odra_location.clone() {
            OdraLocation::Local(local_path) => TemplatePath {
                auto_path: Some(local_path.as_os_str().to_str().unwrap().to_string()),
                subfolder: Some(format!("templates/{}", template)),
                test: false,
                git: None,
                branch: None,
                tag: None,
                path: None,
                favorite: None,
            },
            OdraLocation::Remote(repo, branch) => TemplatePath {
                auto_path: Some(repo),
                subfolder: Some(format!("templates/{}", template)),
                test: false,
                git: None,
                branch,
                tag: None,
                path: None,
                favorite: None,
            },
            OdraLocation::CratesIO(version) => TemplatePath {
                auto_path: Some(ODRA_TEMPLATE_GH_REPO.to_string()),
                subfolder: Some(format!("templates/{}", template)),
                test: false,
                git: None,
                branch: Some(format!("release/{}", version)),
                tag: None,
                path: None,
                favorite: None,
            },
        };
        template_path
    }
}
