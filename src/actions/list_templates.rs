//! Module responsible for listing templates in cli.

use crate::{
    project::OdraLocation,
    template::{TemplateGenerator, TemplateType},
};

/// ListTemplatesAction configuration.
#[derive(Clone)]
pub struct ListTemplatesAction;

impl ListTemplatesAction {
    pub fn list(odra_location: OdraLocation) {
        let templates = TemplateGenerator::new_gh_repo(odra_location).fetch_templates();
        println!("Available contract templates:");
        templates
            .iter()
            .filter(|template| template.template_type == TemplateType::Contract)
            .for_each(|template| {
                println!("  {:<15}{}", template.name, template.description);
            });
        println!("\nAvailable project templates:");
        templates
            .iter()
            .filter(|template| template.template_type == TemplateType::Project)
            .for_each(|template| {
                println!("  {:<15}{}", template.name, template.description);
            });
    }
}
