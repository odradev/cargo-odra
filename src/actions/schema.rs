//! Module for generating contracts schema.

use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use crate::{
    command,
    errors::Error,
    fingerprint,
    log,
    odra_toml::Contract,
    paths,
    project::Project,
    utils,
};

/// SchemaAction configuration.
pub struct SchemaAction<'a> {
    project: &'a Project,
    contracts_names: Option<String>,
}

impl<'a> SchemaAction<'a> {
    /// Crate a new SchemaAction for a given configuration.
    pub fn new(project: &'a Project, contracts_names: Option<String>) -> Self {
        SchemaAction {
            project,
            contracts_names,
        }
    }
}

impl SchemaAction<'_> {
    /// Main function that runs the whole workflow.
    pub fn build(&self) {
        utils::check_target_requirements();
        utils::validate_contract_name_argument(self.project, self.contracts_names());
        utils::validate_contract_names(self.project);
        self.generate_schema_files();
    }

    /// Generates *_schema.json files, skipping the contracts whose schema is up to date (see the
    /// `fingerprint` module).
    fn generate_schema_files(&self) {
        log::info("Generating schema files...");
        let contracts =
            utils::contracts(self.project, self.contracts_names()).unwrap_or_else(|_| {
                Error::FailedToParseArgument("contracts_names".to_string()).print_and_die()
            });
        utils::validate_external_contracts(self.project, &contracts);

        let project_root = self.project.project_root();
        let target_dir = paths::build_target_dir(&project_root);
        let environment = fingerprint::Environment::detect();
        let total = contracts.len();
        let (fresh, stale): (Vec<_>, Vec<_>) = contracts.into_iter().partition(|contract| {
            find_contract_schema(&project_root, &contract.struct_name()).is_some_and(|schema| {
                fingerprint::is_fresh(
                    &self.fingerprint_path(&target_dir, contract),
                    &schema,
                    &self.inputs(contract, &environment),
                )
            })
        });
        if !fresh.is_empty() {
            log::info(format!(
                "{} of {total} schema files are up to date: {}",
                fresh.len(),
                fresh
                    .iter()
                    .map(Contract::struct_name)
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }

        for contract in stale {
            let host_crate = contract.host_crate(self.project);
            command::cargo_generate_schema_files(
                project_root.clone(),
                &contract.odra_module_value(self.project),
                &host_crate.name,
            );
            let Some(schema) = find_contract_schema(&project_root, &contract.struct_name()) else {
                log::warn(format!(
                    "No schema file names {} under {}; it will be generated again next time.",
                    contract.struct_name(),
                    schemas_dir(&project_root).display()
                ));
                continue;
            };
            let mut extra_files = fingerprint::manifests(self.project, &host_crate.root);
            extra_files.push(schema);
            fingerprint::record_or_warn(
                &self.fingerprint_path(&target_dir, &contract),
                &paths::schema_dep_info_path_in_target_dir(&target_dir, &host_crate.name),
                &extra_files,
                self.inputs(&contract, &environment),
            );
        }
    }

    fn fingerprint_path(&self, target_dir: &Path, contract: &Contract) -> PathBuf {
        fingerprint::path(
            target_dir,
            &self.project.project_root(),
            &contract.struct_name(),
            fingerprint::Output::Schema,
        )
    }

    /// The environment a contract's schema is generated under, for its fingerprint.
    fn inputs(
        &self,
        contract: &Contract,
        environment: &fingerprint::Environment,
    ) -> BTreeMap<String, String> {
        environment.inputs(&[
            ("odra-module", &contract.odra_module_value(self.project)),
            ("package", &contract.host_crate(self.project).package),
        ])
    }

    fn contracts_names(&self) -> String {
        self.contracts_names.clone().unwrap_or_default()
    }
}

/// The directory the project's `_build_schema` binary writes the schema files to, when run
/// from `project_root`.
fn schemas_dir(project_root: &Path) -> PathBuf {
    project_root
        .join("resources")
        .join("casper_contract_schemas")
}

/// The schema file of `contract_name` in the project at `project_root`: the JSON file under
/// `resources/casper_contract_schemas` whose `contract_name` is that name.
///
/// The file is found by its content and not by its name, because the name is derived by
/// `odra-build` with a snake-casing of its own (`RejectingNFTReceiver` gives
/// `rejectingnft_receiver_schema.json`), and guessing it wrong would only mean regenerating the
/// schema at every run.
fn find_contract_schema(project_root: &Path, contract_name: &str) -> Option<PathBuf> {
    let entries = std::fs::read_dir(schemas_dir(project_root)).ok()?;
    let mut paths: Vec<PathBuf> = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
        .collect();
    paths.sort();
    paths.into_iter().find(|path| {
        std::fs::read_to_string(path)
            .ok()
            .and_then(|content| serde_json::from_str::<serde_json::Value>(&content).ok())
            .is_some_and(|schema| schema["contract_name"] == contract_name)
    })
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::find_contract_schema;

    #[test]
    fn finds_a_schema_file_by_the_contract_name_inside_it() {
        let root = std::env::temp_dir().join(format!("cargo-odra-schemas-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let dir = root.join("resources/casper_contract_schemas");
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join("rejectingnft_receiver_schema.json"),
            r#"{"casper_contract_schema_version": 1, "contract_name": "RejectingNFTReceiver"}"#,
        )
        .unwrap();
        fs::write(
            dir.join("erc20_schema.json"),
            r#"{"contract_name": "Erc20"}"#,
        )
        .unwrap();
        fs::write(dir.join("notes.txt"), "contract_name: Erc20").unwrap();

        assert_eq!(
            find_contract_schema(&root, "RejectingNFTReceiver"),
            Some(dir.join("rejectingnft_receiver_schema.json"))
        );
        assert_eq!(
            find_contract_schema(&root, "Erc20"),
            Some(dir.join("erc20_schema.json"))
        );
        assert_eq!(find_contract_schema(&root, "Cep18"), None);
        assert_eq!(find_contract_schema(&root.join("nowhere"), "Erc20"), None);

        let _ = fs::remove_dir_all(&root);
    }
}
