//! Module for managing and building backends
use std::fs::File;
use std::io::Write;

use std::fs;

use cargo_toml::{Dependency, DependencyDetail, DepsSet};

use crate::cargo_toml::{odra_dependency, project_name};
use crate::consts;
use crate::errors::Error;
use crate::odra_toml::OdraToml;
use crate::paths::BuilderPaths;
use crate::{command, log};

#[derive(Debug)]
/// Backend configuration
pub struct BuildAction {
    backend: String,
    builder_paths: BuilderPaths,
    odra_toml: OdraToml,
}

/// Getters
impl BuildAction {
    pub fn new(backend: String) -> Self {
        BuildAction {
            backend: backend.clone(),
            builder_paths: BuilderPaths::new(backend),
            odra_toml: OdraToml::load(),
        }
    }

    /// Returns the name of the backend.
    /// It is also the name of the Odra's feature.
    pub fn backend_name(&self) -> String {
        self.backend.clone()
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
        log::info(format!(
            "Preparing {} builder in {} directory...",
            self.backend_name(),
            self.builder_paths.root_as_string()
        ));

        self.create_builder_directories();
        self.create_builder_cargo_toml();
        self.create_build_files();
    }

    // TODO replace with mkdir.
    fn create_builder_directories(&self) {
        if !self.builder_paths.root().is_dir() {
            fs::create_dir(self.builder_paths.root()).unwrap();
        }
        if !self.builder_paths.src().is_dir() {
            fs::create_dir(self.builder_paths.src()).unwrap();
        }
    }

    fn create_builder_cargo_toml(&self) {
        crate::cargo_toml::builder_cargo_toml(
            &self.builder_paths,
            self.builder_dependencies(),
            &self.odra_toml,
        );
    }

    fn create_build_files(&self) {
        for contract in self.odra_toml.contracts.iter() {
            let path = self.builder_paths.wasm_build(&contract.name);
            if !path.exists() {
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
        for contract in self.odra_toml.contracts.clone().iter() {
            command::cargo(
                self.builder_paths.root_as_string(),
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
        for contract in self.odra_toml.contracts.iter() {
            // TODO Move to command.rs
            command::cargo(
                self.builder_paths.root_as_string(),
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
        for contract in self.odra_toml.contracts.iter() {
            let source = format!(
                "{}/target/wasm32-unknown-unknown/release/{}.wasm",
                self.builder_paths.root_as_string(),
                contract.name
            );
            let target = format!("wasm/{}.wasm", contract.name);

            log::info(format!("Saving {}", target));

            command::cp(&source, &target);
        }
    }

    fn optimize_wasm_files(&self) {
        for contract in self.odra_toml.contracts.iter() {
            command::wasm_strip(&contract.name);
        }
    }

    fn fmt(&self) {
        command::fmt(&self.builder_paths.root_as_string());
    }

    /// Returns a set of dependencies used by backend.
    pub fn builder_dependencies(&self) -> DepsSet {
        let mut dependencies = DepsSet::new();
        dependencies.insert(consts::ODRA_CRATE_NAME.to_string(), self.odra_dependency());
        dependencies.insert(project_name(), self.project_dependency());
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
