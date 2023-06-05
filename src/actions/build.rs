//! Module for managing and building backends.

use std::{path::Path, ops::RangeBounds};

use cargo_toml::{Dependency, DependencyDetail, DepsSet};

use crate::{
    cargo_toml::odra_dependency,
    command,
    consts::ODRA_TEMPLATE_GH_RAW_REPO,
    errors::Error,
    log,
    odra_toml::Contract,
    paths::{self, BuilderPaths},
    project::Project,
    template::TemplateGenerator,
};

/// BuildAction configuration.
pub struct BuildAction<'a> {
    backend: String,
    contracts_names: Vec<String>,
    builder_paths: BuilderPaths,
    project: &'a Project,
    template_generator: TemplateGenerator,
}

/// BuildAction implementation.
impl<'a> BuildAction<'a> {
    /// Crate a new BuildAction for a given backend.
    pub fn new(project: &'a Project, backend: String, contracts_names: Vec<String>) -> Self {
        BuildAction {
            backend: backend.clone(),
            contracts_names,
            builder_paths: BuilderPaths::new(backend, project.project_root.clone()),
            project,
            template_generator: TemplateGenerator::new(
                ODRA_TEMPLATE_GH_RAW_REPO.to_string(),
                project.project_odra_location(),
            ),
        }
    }
}

impl BuildAction<'_> {
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
            dependencies.insert(member.name.clone(), self.project_dependency(&member.root));
        });
        dependencies
    }

    /// Main function that runs the whole workflow for a backend.
    pub fn build(&self) {
        self.check_target_requirements();
        self.validate_contract_name_argument();
        self.prepare_builder();
        self.build_wasm_sources();
        self.format_builder_files();
        // self.build_wasm_files();
        // self.copy_wasm_files();
        // self.optimize_wasm_files();
    }

    /// Returns list of contract to process.
    fn contracts(&self) -> Vec<Contract> {
        let odra_toml = self.project.odra_toml();
        if self.contracts_names.is_empty() {
            odra_toml.contracts
        } else {
            odra_toml
                .contracts
                .into_iter()
                .filter(|c| self.contracts_names.contains(&c.name))
                .collect()
        }
    }

    /// Check if wasm32-unknown-unknown target is installed.
    fn check_target_requirements(&self) {
        if !command::command_output("rustup target list --installed")
            .contains("wasm32-unknown-unknown")
        {
            Error::WasmTargetNotInstalled.print_and_die();
        }
    }

    /// Check if contract name argument is valid if set.
    fn validate_contract_name_argument(&self) {
        self.contracts_names.iter().for_each(|contract_name| {
            if !self
                .project
                .odra_toml()
                .contracts
                .iter()
                .any(|c| c.name == *contract_name)
            {
                Error::ContractNotFound(contract_name.clone()).print_and_die();
            }
        });
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
            self.contracts(),
        );

        // Build files.
        self.create_default_build_file();
    }

    /// Prepare _build.rs files.
    fn create_build_files(&self) {
        for contract in self.contracts() {
            let path = self.builder_paths.wasm_build(&contract.name);
            if !path.exists() {
                let content = self.template_generator.wasm_source_builder(
                    &contract.fqn,
                    &contract.name,
                    &self.backend_name(),
                );
                command::write_to_file(path, &content);
            }
        }
    }

    fn create_default_build_file(&self) {
        self.create_build_file(&self.contracts());
    }

    fn create_build_file(&self, contracts: &[Contract]) {
        let path = self.builder_paths.wasm_build("contracts");
        if path.exists() {
            return;
        }
        
        let contract_modules: Vec<_> = contracts
            .iter()
            .map(|contract| format!(
                "mod {} {{ odra::{}::codegen::gen_contract!({}, \"{}\"); }}", 
                contract.name, 
                &self.backend_name(), 
                contract.fqn, 
                contract.name
            ))
            .collect();

        let mod_matching: Vec<_> = contracts
            .iter()
            .map(|contract| format!(
                "Some(\"{}\") => {}::main(),", 
                contract.name, 
                contract.name
            ))
            .collect();
        let mod_matching: String = mod_matching.join("\n");

        let main = format!(r#"
            fn main() {{
                let args: Vec<String> = std::env::args().collect();
                match args.get(1).map(String::as_str) {{
                    {}
                    _ => println!("Please provide a valid module name!"),
                }}
            }}
        "#, mod_matching);

        let content = format!("{}\n{}", contract_modules.join("\n"), main);
        
        // let content = format!("odra::{}::codegen::gen_contract!({});", &self.backend_name(), contracts);
        command::write_to_file(path, &content);
    }

    /// Prepare _wasm.rs file.
    fn build_wasm_sources(&self) {
        log::info("Generating _wasm.rs files...");
        for contract in self.contracts() {
            command::cargo_build_wasm_sources(self.builder_paths.root(), &contract.name);
        }
    }

    /// Build _wasm.rs files into .wasm files.
    fn build_wasm_files(&self) {
        log::info("Generating wasm files...");
        for contract in self.contracts() {
            command::cargo_build_wasm_files(self.builder_paths.root(), &contract.name);
        }
    }

    /// Copy *.wasm files into wasm directory.
    fn copy_wasm_files(&self) {
        log::info("Copying wasm files...");
        command::mkdir(paths::wasm_dir(self.project.project_root()));
        for contract in self.contracts() {
            let source = paths::wasm_path_in_target(&contract.name, self.project.project_root());
            let target = paths::wasm_path_in_wasm_dir(&contract.name, self.project.project_root());
            log::info(format!("Saving {}", target.display()));
            command::cp(source, target);
        }
    }

    /// Run wasm-strip on *.wasm files in wasm directory.
    fn optimize_wasm_files(&self) {
        log::info("Optimizing wasm files...");
        for contract in self.contracts() {
            command::wasm_strip(&contract.name, self.project.project_root());
        }
    }

    /// Format Rust files in builder directory.
    fn format_builder_files(&self) {
        command::cargo_fmt(self.builder_paths.root());
    }

    /// Returns Odra dependency tailored for use by builder.
    fn odra_dependency(&self) -> Dependency {
        let first_member = self.project.members.first().unwrap();
        match odra_dependency(&first_member.cargo_toml) {
            Dependency::Simple(simple) => Dependency::Detailed(DependencyDetail {
                version: Some(simple),
                ..Default::default()
            }),
            Dependency::Detailed(mut odra_details) => {
                odra_details.features = vec![self.backend_name()];
                odra_details.default_features = false;
                if odra_details.path.is_some() {
                    if self.project.is_workspace() {
                        odra_details.path = Some(odra_details.path.unwrap());
                    } else {
                        odra_details.path = Some(format!("../{}", odra_details.path.unwrap()));
                    }
                }
                Dependency::Detailed(odra_details)
            }
            Dependency::Inherited(_) => {
                Error::NotImplemented("Inherited dependencies are not supported yet.".to_string())
                    .print_and_die();
            }
        }
    }

    /// Returns project dependency with specific backend feature enabled.
    fn project_dependency(&self, location: &Path) -> Dependency {
        Dependency::Detailed(DependencyDetail {
            path: Some(
                location
                    .to_path_buf()
                    .into_os_string()
                    .to_str()
                    .unwrap()
                    .to_string(),
            ),
            features: vec![self.backend_name()],
            default_features: false,
            ..Default::default()
        })
    }
}
