use std::path::PathBuf;

use crate::{
    actions::schema::SchemaAction,
    cargo_toml,
    command::{self, mkdir},
    errors::Error,
    project::Project,
    template::{TemplateGenerator, TemplateType},
};

/// GenerateClientAction configuration.
pub struct GenerateClientAction<'a> {
    project: &'a mut Project,
    module_root: PathBuf,
}

impl<'a> GenerateClientAction<'a> {
    pub fn new(project: &'a mut Project) -> Self {
        let name = format!("{}_client", project.project_crate_name());
        let module_root = project.project_root().join(name);
        GenerateClientAction {
            project,
            module_root,
        }
    }

    /// Create a new GenerateClientAction for a given contract.
    pub fn generate(&mut self) {
        println!("Generate client code...");
        match self.project.add_client_if_needed() {
            Ok(_) => {
                self.create_crate_structure();
                self.write_cargo_toml();
                self.write_main_rs();
                self.generate_schema();
                self.build_client();
            }
            Err(Error::ClientAlreadyExists) => {
                self.generate_schema();
                self.build_client();
            }
            Err(e) => e.print_and_die(),
        }
    }

    fn create_crate_structure(&self) {
        let path = self.module_root.clone();
        mkdir(path.clone());
        mkdir(path.join("src"));
    }

    fn write_cargo_toml(&self) {
        let odra_location = self.project.project_odra_location();
        let template_generator = TemplateGenerator::new_gh_repo(odra_location.clone());

        let templates = template_generator.fetch_templates();
        let client_template = templates
            .iter()
            .find(|template| template.template_type == TemplateType::Client && template.name.contains("cargo"))
            .unwrap_or_else(|| {
                Error::TemplateNotFound("client".to_string()).print_and_die();
            });

        let core = format!(
            "odra-core = {{ {} }}",
            cargo_toml::odra_project_dependency_string(&odra_location, "core", false)
        );
        let wasm_client = format!(
            "odra-wasm-client = {{ {} }}",
            cargo_toml::odra_project_dependency_string(&odra_location, "odra-wasm-client", false)
        );
        let wasm_client_builder = format!(
            "odra-wasm-client-builder = {{ {} }}",
            cargo_toml::odra_project_dependency_string(&odra_location, "odra-wasm-client-builder", false)
        );
        let client_template = template_generator
            .fetch_template(&client_template.name)
            .replace("#odra_core_dependency", &core)
            .replace("#odra_wasm_client_dependency", &wasm_client)
            .replace("#odra_wasm_client_builder_dependency", &wasm_client_builder)
            .replace("{{project-name}}", &self.project.name);

        let cargo_toml_path = self.module_root.join("Cargo.toml");
        command::write_to_file(cargo_toml_path, &client_template);
    }

    fn write_main_rs(&self) {
        let odra_location = self.project.project_odra_location();
        let template_generator = TemplateGenerator::new_gh_repo(odra_location.clone());

        let templates = template_generator.fetch_templates();
        let client_template = templates
            .iter()
            .find(|template| template.template_type == TemplateType::Client && template.name.contains("codegen"))
            .unwrap_or_else(|| {
                Error::TemplateNotFound("client".to_string()).print_and_die();
            });

        let client_template = template_generator
            .fetch_template(&client_template.name)
            .replace("{{project-name}}", &self.project.name);

        let main_rs_path = self.module_root.join("src").join("main.rs");
        command::write_to_file(main_rs_path, &client_template);
    }

    fn generate_schema(&self) {
        SchemaAction::new(self.project, None).build();
    }

    fn build_client(&self) {
        // BuildAction::new(self.project).build_client();
    }
}
