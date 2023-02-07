//! Module for managing and building backends.

use cargo_toml::{Dependency, DependencyDetail, DepsSet, PatchSet};

use crate::{
    cargo_toml::{odra_dependency, project_name},
    command,
    errors::Error,
    log,
    odra_toml::OdraToml,
    paths::{self, BuilderPaths},
    template,
};

/// BuildAction configuration.
pub struct BuildAction {
    backend: String,
    builder_paths: BuilderPaths,
    odra_toml: OdraToml,
}

/// BuildAction implementation.
impl BuildAction {
    /// Crate a new BuildAction for a given backend.
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

    /// Returns a set of dependencies used by backend.
    pub fn builder_dependencies(&self) -> DepsSet {
        let mut dependencies = DepsSet::new();
        dependencies.insert(String::from("odra"), self.odra_dependency());
        dependencies.insert(project_name(), self.project_dependency());
        dependencies
    }

    /// This function creates a PatchSet and inserts a DepsSet with the key "crates-io" 
    /// and value of the function call self.casper_types_dependency().
    pub fn builder_patch(&self) -> PatchSet {
        let mut patch_set = PatchSet::new();
        let mut crates_io_patch_deps = DepsSet::new();
        crates_io_patch_deps.insert("casper-types".to_string(), self.casper_types_dependency());
        patch_set.insert("crates-io".to_string(), crates_io_patch_deps);
        patch_set
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

        // Prepare directories.
        command::mkdir(self.builder_paths.src());

        // Build Cargo.toml
        crate::cargo_toml::builder_cargo_toml(
            &self.builder_paths,
            self.builder_dependencies(),
            self.builder_patch(),
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

    /// This function creates a Dependency object with 
    /// the git repository set to "https://github.com/kpob/casper-node" 
    /// and the branch set to "feature/k256-update". 
    fn casper_types_dependency(&self) -> Dependency {
        Dependency::Detailed(DependencyDetail { 
            git: Some("https://github.com/kpob/casper-node".to_string()), 
            branch: Some("feature/k256-update".to_string()),
            ..Default::default() 
        })
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
