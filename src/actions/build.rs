//! Module for managing and building backends
use std::fs::File;
use std::io::Write;
use std::path::Path;

use std::fs;

use cargo_toml::{Dependency, DependencyDetail, DepsSet};
use serde_derive::{Deserialize, Serialize};

use crate::cargo_toml::odra_dependency;
use crate::consts;
use crate::errors::Error;
use crate::odra_toml::OdraToml;
use crate::{command, log};

#[derive(Deserialize, Serialize, Debug, Clone)]
/// Backend configuration
pub struct BuildAction {
    backend: String,
    odra_toml: OdraToml,
}

/// Getters
impl BuildAction {
    pub fn new(backend: String) -> Self {
        BuildAction {
            backend,
            odra_toml: OdraToml::load(),
        }
    }

    /// Returns the name of the backend.
    /// It is also the name of the Odra's feature.
    pub fn backend_name(&self) -> String {
        self.backend.clone()
    }

    /// Returns a path where builder lives
    pub fn builder_path(&self) -> String {
        format!(".builder_{}/", self.backend_name())
    }

    /// Main function that runs the whole workflow for backend
    pub fn build(&self) {
        self.check_target_requirements();
        self.prepare_builder();
        self.build_wasm_sources();
        self.build_wasm_files();
        self.fmt();
        self.copy_wasm_files();
        self.optimize_wasm_files();
    }

    /// Check if wasm32-unknown-unknown target is installed.
    fn check_target_requirements(&self) {
        if !command::command_output("rustup target list --installed")
            .contains("wasm32-unknown-unknown")
        {
            Error::WasmTargetNotInstalled.print_and_die();
        }
    }

    fn prepare_builder(&self) {
        log::info(&format!(
            "Preparing {} builder in {} directory...",
            self.backend_name(),
            self.builder_path()
        ));

        self.create_builder_directories();
        self.create_builder_cargo_toml();
        self.create_build_files();
    }

    fn create_builder_directories(&self) {
        // TODO: Cleanup paths.
        if !Path::new(&self.builder_path()).is_dir() {
            fs::create_dir(self.builder_path()).unwrap();
        }
        let src_path = Path::new(&self.builder_path()).join("src");
        if !src_path.is_dir() {
            fs::create_dir(src_path).unwrap();
        }
    }

    fn create_builder_cargo_toml(&self) {
        crate::cargo_toml::builder_cargo_toml(
            self.builder_path(),
            self.builder_dependencies(),
            &self.odra_toml,
        );
    }

    fn create_build_files(&self) {
        for (_, contract) in self.odra_toml.contracts.iter() {
            let path = self.builder_path() + contract.path.as_str();
            if !Path::new(&path).exists() {
                // TODO: Hide replaces.
                let contents = consts::DEF_RS
                    .replace("#contract_fqn", &contract.fqn)
                    .replace("#contract_name", &contract.name)
                    .replace("#backend_name", &self.backend_name());
                let mut file = File::create(path).unwrap();
                file.write_all(contents.as_bytes()).unwrap();
            }
        }
    }

    fn build_wasm_sources(&self) {
        log::info("Generating _wasm.rs files...");
        for (_, contract) in self.odra_toml.contracts.clone().iter() {
            command::cargo(
                self.builder_path(),
                vec![
                    "run",
                    "--bin",
                    format!("{}_build", &contract.name).as_str(),
                    "--no-default-features",
                ],
            );
        }
    }

    fn build_wasm_files(&self) {
        log::info("Generating wasm files...");
        for (_, contract) in self.odra_toml.contracts.iter() {
            // TODO Move to command.rs
            command::cargo(
                self.builder_path(),
                vec![
                    "build",
                    "--target",
                    "wasm32-unknown-unknown",
                    "--bin",
                    &contract.name,
                    "--release",
                    "--no-default-features",
                ],
            );
        }
    }

    fn copy_wasm_files(&self) {
        log::info("Copying wasm files...");
        command::mkdir("target/debug");
        command::mkdir("wasm");
        for (_, contract) in self.odra_toml.contracts.iter() {
            let source = format!(
                "{}target/wasm32-unknown-unknown/release/{}.wasm",
                self.builder_path(),
                contract.name
            );
            let target = format!("wasm/{}.wasm", contract.name);

            log::info(&format!("Saving {}", target));

            command::cp(&source, &target);
        }
    }

    fn optimize_wasm_files(&self) {
        for (_, contract) in self.odra_toml.contracts.iter() {
            command::wasm_strip(&contract.name);
        }
    }

    fn fmt(&self) {
        command::fmt(&self.builder_path());
    }

    /// Returns a set of dependencies used by backend
    pub fn builder_dependencies(&self) -> DepsSet {
        let mut dependencies = DepsSet::new();
        dependencies.insert(consts::ODRA_CRATE_NAME.to_string(), self.odra_dependency());
        dependencies.insert(self.odra_toml.name.clone(), self.project_dependency());
        dependencies
    }

    /// Returns Odra dependency tailored for use by builder.
    fn odra_dependency(&self) -> Dependency {
        match odra_dependency() {
            Dependency::Simple(simple) => Dependency::Detailed(DependencyDetail {
                version: Some(simple),
                ..Default::default()
            }),
            Dependency::Detailed(mut odra_details) => {
                odra_details.features = vec![self.backend_name()];
                odra_details.default_features = Some(false);
                if odra_details.path.is_some() {
                    odra_details.path = Some(format!("../{}", odra_details.path.unwrap()));
                }
                Dependency::Detailed(odra_details)
            }
        }
    }

    /// Returns project dependency with specific feature enabled.
    fn project_dependency(&self) -> Dependency {
        Dependency::Detailed(DependencyDetail {
            path: Some("..".to_string()),
            features: vec![self.backend_name()],
            default_features: Some(false),
            ..Default::default()
        })
    }
}
