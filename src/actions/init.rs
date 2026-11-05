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
    log,
    paths,
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

        let project_path = cargo_generate::generate(Self::generate_args(
            template_path,
            paths::to_snake_case(&init_command.name),
            init,
            None,
        ))
        .unwrap_or_else(|e| {
            Error::FailedToGenerateProjectFromTemplate(e.to_string()).print_and_die();
        });

        let project_name = init_command.name.to_lowercase();
        rename_file(project_path, &project_name).unwrap_or_else(|err| err.print_and_die());

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

        rename_file(cargo_toml_path, "Cargo.toml").unwrap_or_else(|err| err.print_and_die());
        log::info("Done!");
    }

    /// Arguments for `cargo_generate::generate`. `destination` is `None` in production: the
    /// project is generated in the current directory.
    fn generate_args(
        template_path: TemplatePath,
        name: String,
        init: bool,
        destination: Option<PathBuf>,
    ) -> GenerateArgs {
        GenerateArgs {
            template_path,
            list_favorites: false,
            name: Some(name),
            force: true,
            verbose: false,
            quiet: false,
            continue_on_error: false,
            template_values_file: None,
            silent: false,
            config: None,
            vcs: Some(Vcs::Git),
            lib: false,
            bin: false,
            ssh_identity: None,
            gitconfig: None,
            define: vec![format!("date={}", Utc::now().format("%Y-%m-%d"))],
            init,
            destination,
            force_git_init: false,
            allow_commands: false,
            overwrite: false,
            skip_submodules: true,
            // cargo-generate 0.24 adds the generated project to the `members` of the nearest
            // enclosing Cargo workspace and rewrites that workspace's Cargo.toml - dropping
            // entries it does not model and reformatting the rest. An Odra project is a
            // standalone project (or its own workspace), never a member of whatever workspace
            // happens to sit above the directory it is created in, so this is always off.
            no_workspace: true,
            other_args: None,
        }
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
        )
        .unwrap_or_else(|err| err.print_and_die());
    }

    fn assert_dir_is_empty(dir: PathBuf) {
        if dir.read_dir().unwrap().next().is_some() {
            Error::CurrentDirIsNotEmpty.print_and_die();
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    /// A parent workspace like Odra's own: the directory the project is generated in is
    /// excluded, and the exclude entry names the directory, not the project inside it.
    const PARENT_WORKSPACE: &str = r#"[workspace]
exclude = [
  "templates/full",
  "tests",
]
members = ["core"]
resolver = "2"

# A comment that a rewrite would drop.
[workspace.package]
edition = "2021"
"#;

    #[test]
    fn new_project_is_not_added_to_an_enclosing_workspace() {
        let root =
            std::env::temp_dir().join(format!("cargo-odra-init-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);

        let template = root.join("template");
        fs::create_dir_all(&template).unwrap();
        fs::write(
            template.join("Cargo.toml"),
            "[package]\nname = \"{{project-name}}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        )
        .unwrap();

        let parent = root.join("parent");
        let destination = parent.join("tests");
        fs::create_dir_all(&destination).unwrap();
        fs::write(parent.join("Cargo.toml"), PARENT_WORKSPACE).unwrap();

        let template_path = TemplatePath {
            path: Some(template.to_str().unwrap().to_string()),
            ..TemplatePath::default()
        };
        let project = cargo_generate::generate(InitAction::generate_args(
            template_path,
            "my_project".to_string(),
            false,
            Some(destination.clone()),
        ))
        .unwrap();

        assert_eq!(project, destination.join("my_project"));
        assert!(project.join("Cargo.toml").exists());
        assert_eq!(
            fs::read_to_string(parent.join("Cargo.toml")).unwrap(),
            PARENT_WORKSPACE
        );

        let _ = fs::remove_dir_all(&root);
    }
}
