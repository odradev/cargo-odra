//! Module responsible for initializing an Odra project.

use std::path::{Path, PathBuf};

use cargo_generate::{GenerateArgs, TemplatePath, Vcs};
use chrono::Utc;

use crate::{
    cargo_toml,
    cli::InitCommand,
    command::{rename_file, replace_in_file},
    consts::{ODRA_TEMPLATE_GH_RAW_REPO, ODRA_TEMPLATE_GH_REPO},
    errors::Error,
    log, paths,
    project::OdraLocation,
    template::TemplateGenerator,
};

/// InitAction configuration.
#[derive(Clone)]
pub struct InitAction;

/// InitAction implementation.
impl InitAction {
    pub fn generate_project(init_command: InitCommand, current_dir: PathBuf, init: bool) {
        if init {
            Self::assert_dir_is_empty(current_dir.clone());
        }

        log::info("Generating a new project...");

        let odra_location = OdraLocation::from_source(init_command.source);

        let template_repository_path =
            TemplateGenerator::new(ODRA_TEMPLATE_GH_RAW_REPO.to_string(), odra_location.clone())
                .find_template(&init_command.template)
                .path;

        let template_path = match odra_location.clone() {
            OdraLocation::Local(local_path) => TemplatePath {
                auto_path: Some(local_path.as_os_str().to_str().unwrap().to_string()),
                subfolder: Some(template_repository_path),
                test: false,
                git: None,
                branch: None,
                tag: None,
                revision: None,
                path: None,
                favorite: None,
            },
            OdraLocation::Remote(repo, branch) => TemplatePath {
                auto_path: Some(repo),
                subfolder: Some(template_repository_path),
                test: false,
                git: None,
                branch,
                tag: None,
                revision: None,
                path: None,
                favorite: None,
            },
            OdraLocation::CratesIO(version) => TemplatePath {
                auto_path: Some(ODRA_TEMPLATE_GH_REPO.to_string()),
                subfolder: Some(template_repository_path),
                test: false,
                git: None,
                branch: Some(format!("release/{version}")),
                tag: None,
                revision: None,
                path: None,
                favorite: None,
            },
        };

        let project_path = cargo_generate::generate(GenerateArgs {
            template_path,
            list_favorites: false,
            name: Some(paths::to_snake_case(&init_command.name)),
            force: true,
            verbose: false,
            template_values_file: None,
            silent: false,
            config: None,
            vcs: Some(Vcs::Git),
            lib: false,
            bin: false,
            ssh_identity: None,
            define: vec![format!("date={}", Utc::now().format("%Y-%m-%d"))],
            init,
            destination: None,
            force_git_init: false,
            allow_commands: false,
            overwrite: false,
            skip_submodules: true,
            other_args: None,
        })
        .unwrap_or_else(|e| {
            Error::FailedToGenerateProjectFromTemplate(e.to_string()).print_and_die();
        });

        let project_name = init_command.name.to_lowercase();
        rename_file(project_path, &project_name);

        let mut cargo_toml_path = current_dir;
        if !init {
            cargo_toml_path.push(project_name);
        }
        cargo_toml_path.push("_Cargo.toml");

        Self::replace_package_placeholder(
            init,
            &odra_location,
            &cargo_toml_path,
            "#odra_dependency",
            "odra",
            "odra",
        );

        Self::replace_package_placeholder(
            init,
            &odra_location,
            &cargo_toml_path,
            "#odra_test_dependency",
            "odra-test",
            "odra-test",
        );

        Self::replace_package_placeholder(
            init,
            &odra_location,
            &cargo_toml_path,
            "#odra_build_dependency",
            "odra-build",
            "odra-build",
        );

        Self::replace_package_placeholder(
            init,
            &odra_location,
            &cargo_toml_path,
            "#odra_modules_dependency",
            "odra-modules",
            "modules",
        );

        Self::replace_package_placeholder(
            init,
            &odra_location,
            &cargo_toml_path,
            "#odra_cli_dependency",
            "odra-cli",
            "odra-cli",
        );

        rename_file(cargo_toml_path, "Cargo.toml");
        log::info("Done!");
    }

    fn replace_package_placeholder(
        init: bool,
        odra_location: &OdraLocation,
        cargo_toml_path: &Path,
        placeholder: &str,
        crate_name: &str,
        crate_path: &str,
    ) {
        replace_in_file(
            cargo_toml_path.to_path_buf(),
            placeholder,
            format!(
                "{} = {{ {} }}",
                crate_name,
                cargo_toml::odra_project_dependency_string(odra_location, crate_path, init)
            )
            .as_str(),
        );
    }

    fn assert_dir_is_empty(dir: PathBuf) {
        if dir.read_dir().unwrap().next().is_some() {
            Error::CurrentDirIsNotEmpty.print_and_die();
        }
    }
}
