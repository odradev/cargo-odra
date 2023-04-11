//! Module for managing and building backends.

use std::path::PathBuf;
use cargo_toml::{Dependency, DependencyDetail, DepsSet};

use crate::cargo_toml::members;
use crate::{
    cargo_toml::{odra_dependency, project_name},
    command,
    errors::Error,
    log,
    odra_toml::OdraToml,
    paths::{self, BuilderPaths},
    template,
};
use crate::project::Project;

/// BuildAction configuration.
pub struct BuildAction {
    backend: String,
    odra_toml: OdraToml,
    builder_paths: BuilderPaths,
    project: Project,
}

/// BuildAction implementation.
impl BuildAction {
    /// Crate a new BuildAction for a given backend.
    pub fn new(backend: String) -> Self {
        let project = Project::detect();
        BuildAction {
            backend: backend.clone(),
            odra_toml: OdraToml::load().unwrap(),
            builder_paths: BuilderPaths::new(backend),
            project,
        }
    }

    /// Returns the name of the backend.
    /// It is also the name of the Odra's feature.
    pub fn backend_name(&self) -> String {
        self.backend.clone()
    }

    /// Returns a set of dependencies used by backend.
    pub fn builder_dependencies(&self) -> DepsSet {
        let mut dependencies = DepsSet::new();
        dependencies.insert(String::from("odra"), self.odra_dependency());
        self.project.members.iter().for_each(|member| {
            dependencies.insert(member.name.clone(), self.project_dependency(&member.name));
        });
        dependencies
    }

    /// Main function that runs the whole workflow for a backend.
    pub fn build(&self) {
        self.check_target_requirements();
        self.prepare_builder();
        self.build_wasm_sources();
        self.build_wasm_files();
        self.format_builder_files();
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

    /// Prepare builder directories and all files.
    fn prepare_builder(&self) {
        log::info(format!(
            "Preparing {} builder in {} directory...",
            self.backend_name(),
            self.builder_paths.root().display()
        ));

        command::mkdir(self.builder_paths.src());

        // Build Cargo.toml
        crate::cargo_toml::builder_cargo_toml(
            &self.builder_paths,
            self.builder_dependencies(),
            &self.odra_toml,
        );

        // Build files.
        self.create_build_files();
    }

    /// Prepare _build.rs files.
    fn create_build_files(&self) {
        for contract in self.odra_toml.contracts.iter() {
            let path = self.builder_paths.wasm_build(&contract.name);
            if !path.exists() {
                let content = template::wasm_source_builder(
                    &contract.fqn,
                    &contract.name,
                    &self.backend_name(),
                );
                command::write_to_file(path, &content);
            }
        }
    }

    /// Prepare _wasm.rs file.
    fn build_wasm_sources(&self) {
        log::info("Generating _wasm.rs files...");
        for contract in self.odra_toml.contracts.iter() {
            command::cargo_build_wasm_sources(self.builder_paths.root(), &contract.name);
        }
    }

    /// Build _wasm.rs files into .wasm files.
    fn build_wasm_files(&self) {
        log::info("Generating wasm files...");
        for contract in self.odra_toml.contracts.iter() {
            command::cargo_build_wasm_files(self.builder_paths.root(), &contract.name);
        }
    }

    /// Copy *.wasm files into wasm directory.
    fn copy_wasm_files(&self) {
        log::info("Copying wasm files...");
        command::mkdir(paths::wasm_dir());
        for contract in self.odra_toml.contracts.iter() {
            let source = paths::wasm_path_in_target(&contract.name);
            let target = paths::wasm_path_in_wasm_dir(&contract.name);
            log::info(format!("Saving {}", target.display()));
            command::cp(source, target);
        }
    }

    /// Run wasm-strip on *.wasm files in wasm directory.
    fn optimize_wasm_files(&self) {
        for contract in self.odra_toml.contracts.iter() {
            command::wasm_strip(&contract.name);
        }
    }

    /// Format Rust files in builder directory.
    fn format_builder_files(&self) {
        command::cargo_fmt(self.builder_paths.root());
    }

    /// Returns Odra dependency tailored for use by builder.
    fn odra_dependency(&self) -> Dependency {
        let first_member = self.project.members.first().unwrap();
        match odra_dependency(first_member.cargo_toml.clone()) {
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
    fn project_dependency(&self, location: &String) -> Dependency {
        Dependency::Detailed(DependencyDetail {
            path: Some(format!("../{}", location.clone())),
            features: vec![self.backend_name()],
            default_features: Some(false),
            ..Default::default()
        })
    }
}
