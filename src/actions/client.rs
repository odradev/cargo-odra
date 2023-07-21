use std::path::PathBuf;

use crate::{command::replace_in_file, project::Project, template::TemplateGenerator};

pub fn client_action(project: &Project) {
    let odra_location = project.project_odra_location();
    let template_path = TemplateGenerator::odra_template_path("client", &odra_location);
    TemplateGenerator::generate_from_template("client", template_path, false);

    replace_in_file(
        project.project_root().join("client/Cargo.toml"),
        "#project_name",
        &project.name,
    )
}
