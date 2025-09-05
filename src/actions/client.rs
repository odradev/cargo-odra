use std::path::PathBuf;

use convert_case::{Case, Casing};

use crate::{
    actions::schema::SchemaAction,
    cargo_toml,
    command,
    errors::Error,
    log,
    project::{OdraLocation, Project},
    template::{TemplateGenerator, TemplateType},
    utils,
};

/// GenerateClientAction configuration.
pub struct GenerateClientAction<'a> {
    project: &'a mut Project,
    module_root: PathBuf,
    odra_location: OdraLocation,
    template_generator: TemplateGenerator,
}

impl<'a> GenerateClientAction<'a> {
    pub fn new(project: &'a mut Project) -> Self {
        let name = format!("{}_client", project.project_crate_name()).to_case(Case::Snake);
        let module_root = project.project_root().join(name);
        let odra_location = project.project_odra_location();
        let template_generator = TemplateGenerator::new_gh_repo(odra_location.clone());
        GenerateClientAction {
            project,
            module_root,
            odra_location,
            template_generator,
        }
    }

    /// Create a new GenerateClientAction for a given contract.
    pub fn generate(&mut self) -> Result<(), Error> {
        utils::check_wasm_pack();
        log::info("Generate client code...");
        match self.add_to_workspace_if_needed() {
            Ok(_) | Err(Error::ClientAlreadyExists) => {
                self.create_crate_structure()?;
                self.write_cargo_toml()?;
                self.write_main_rs()?;
                self.generate_schema();
                self.build_client();
            }
            Err(e) => return Err(e),
        };
        Ok(())
    }

    fn create_crate_structure(&self) -> Result<(), Error> {
        let path = self.module_root.clone();
        command::mkdir(path.join("src"))?;
        command::write_to_file(path.join("src").join("lib.rs"), "")
    }

    fn write_cargo_toml(&self) -> Result<(), Error> {
        let cargo_toml_path = self.module_root.join("Cargo.toml");
        if cargo_toml_path.exists() {
            return Ok(());
        }

        let templates = self.template_generator.fetch_templates();
        let client_template = templates
            .iter()
            .find(|template| {
                template.template_type == TemplateType::Client && template.name.contains("cargo")
            })
            .ok_or(Error::TemplateNotFound("WASM Client Cargo".to_string()))?;

        let core = format!(
            "odra-core = {{ {} }}",
            cargo_toml::odra_project_dependency_string(&self.odra_location, "core", false)
        );
        let wasm_client = format!(
            "odra-wasm-client = {{ {} }}",
            cargo_toml::odra_project_dependency_string(
                &self.odra_location,
                "odra-wasm-client",
                false
            )
        );
        let wasm_client_builder = format!(
            "odra-wasm-client-builder = {{ {} }}",
            cargo_toml::opt_odra_project_dependency_string(
                &self.odra_location,
                "odra-wasm-client-builder",
                false
            )
        );
        let client_template = self
            .template_generator
            .fetch_template(&client_template.name)
            .replace("#odra_core_dependency", &core)
            .replace("#odra_wasm_client_dependency", &wasm_client)
            .replace("#odra_wasm_client_builder_dependency", &wasm_client_builder)
            .replace("{{project-name}}", &self.project.name.to_case(Case::Kebab))
            .replace("{{lib-name}}", &self.project.name.to_case(Case::Snake));

        command::write_to_file(cargo_toml_path, &client_template)
    }

    fn write_main_rs(&self) -> Result<(), Error> {
        let main_rs_path = self.module_root.join("src").join("main.rs");
        if main_rs_path.exists() {
            return Ok(());
        }

        let templates = self.template_generator.fetch_templates();
        let client_template = templates
            .iter()
            .find(|template| {
                template.template_type == TemplateType::Client && template.name.contains("codegen")
            })
            .ok_or(Error::TemplateNotFound("WASM Client Codegen".to_string()))?;

        let client_template = self
            .template_generator
            .fetch_template(&client_template.name)
            .replace("{{project-name}}", &self.project.name.to_case(Case::Snake));

        command::write_to_file(main_rs_path, &client_template)
    }

    fn generate_schema(&self) {
        SchemaAction::new(self.project, None).build();
    }

    fn build_client(&self) {
        command::cargo_build_wasm_client(
            self.module_root.clone(),
            self.project.project_root.clone(),
        );
    }

    fn add_to_workspace_if_needed(&self) -> Result<(), Error> {
        let cargo_toml_location = self.project.cargo_toml_location.clone();
        let name = format!("{}-client", self.project.project_crate_name()).to_case(Case::Snake);
        let mut cargo_toml = cargo_toml::load_cargo_toml(&cargo_toml_location);
        let client_exists = cargo_toml
            .workspace
            .as_ref()
            .ok_or(Error::NotWorkspace)?
            .exclude
            .iter()
            .any(|member| member == &name);

        if client_exists {
            return Err(Error::ClientAlreadyExists);
        }
        cargo_toml
            .workspace
            .as_mut()
            .map(|workspace| {
                workspace.exclude.push(name);
            })
            .ok_or(Error::ClientCreateFailed)?;

        crate::cargo_toml::save_cargo_toml(cargo_toml_location, &cargo_toml)
    }
}
