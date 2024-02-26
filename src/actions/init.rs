//! Module responsible for initializing an Odra project.

use std::path::PathBuf;

use cargo_generate::{GenerateArgs, TemplatePath, Vcs};
use cargo_toml::{Dependency, DependencyDetail};
use chrono::Utc;
use ureq::serde_json;

use crate::{
    cli::InitCommand,
    command::{rename_file, replace_in_file},
    consts::{ODRA_GITHUB_API_DATA, ODRA_TEMPLATE_GH_REPO},
    errors::Error,
    log,
    paths,
    project::OdraLocation,
};

/// InitAction configuration.
#[derive(Clone)]
pub struct InitAction {}

/// InitAction implementation.
impl InitAction {
    pub fn generate_project(init_command: InitCommand, current_dir: PathBuf, init: bool) {
        if init {
            Self::assert_dir_is_empty(current_dir.clone());
        }

        log::info("Generating a new project...");

        let odra_location = Self::odra_location(init_command.source);

        let template_path = match odra_location.clone() {
            OdraLocation::Local(local_path) => TemplatePath {
                auto_path: Some(local_path.as_os_str().to_str().unwrap().to_string()),
                subfolder: Some(format!("templates/{}", init_command.template)),
                test: false,
                git: None,
                branch: None,
                tag: None,
                path: None,
                favorite: None,
            },
            OdraLocation::Remote(repo, branch) => TemplatePath {
                auto_path: Some(repo),
                subfolder: Some(format!("templates/{}", init_command.template)),
                test: false,
                git: None,
                branch,
                tag: None,
                path: None,
                favorite: None,
            },
            OdraLocation::CratesIO(version) => TemplatePath {
                auto_path: Some(ODRA_TEMPLATE_GH_REPO.to_string()),
                subfolder: Some(format!("templates/{}", init_command.template)),
                test: false,
                git: None,
                branch: Some(format!("release/{}", version)),
                tag: None,
                path: None,
                favorite: None,
            },
        };

        cargo_generate::generate(GenerateArgs {
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
            other_args: None,
        })
        .unwrap_or_else(|e| {
            Error::FailedToGenerateProjectFromTemplate(e.to_string()).print_and_die();
        });

        let cargo_toml_path = match init {
            true => {
                let mut path = current_dir;
                path.push("_Cargo.toml");
                path
            }
            false => {
                let mut path = current_dir;
                path.push(paths::to_snake_case(&init_command.name));
                path.push("_Cargo.toml");
                path
            }
        };

        Self::replace_package_placeholder(
            init,
            &odra_location,
            &cargo_toml_path,
            "#odra_dependency",
            "odra",
        );
        Self::replace_package_placeholder(
            init,
            &odra_location,
            &cargo_toml_path,
            "#odra_test_dependency",
            "odra-test",
        );
        Self::replace_package_placeholder(
            init,
            &odra_location,
            &cargo_toml_path,
            "#odra_build_dependency",
            "odra-build",
        );

        rename_file(cargo_toml_path, "Cargo.toml");
        log::info("Done!");
    }

    fn replace_package_placeholder(
        init: bool,
        odra_location: &OdraLocation,
        cargo_toml_path: &PathBuf,
        placeholder: &str,
        crate_name: &str,
    ) {
        replace_in_file(
            cargo_toml_path.clone(),
            placeholder,
            format!(
                "{} = {{ {} }}",
                crate_name,
                toml::to_string(&Self::odra_project_dependency(
                    odra_location.clone(),
                    crate_name,
                    init
                ))
                .unwrap()
                .trim_end()
                .replace('\n', ", ")
            )
            .as_str(),
        );
    }
    fn assert_dir_is_empty(dir: PathBuf) {
        if dir.read_dir().unwrap().next().is_some() {
            Error::CurrentDirIsNotEmpty.print_and_die();
        }
    }

    fn odra_location(source: Option<String>) -> OdraLocation {
        let source = if let Some(source) = source {
            source
        } else {
            Self::odra_latest_version()
        };

        // location on disk
        let local = PathBuf::from(&source);
        if local.exists() {
            OdraLocation::Local(local)
        } else {
            // version
            let version_regex = regex::Regex::new(r"^\d+\.\d+\.\d+$").unwrap();
            if version_regex.is_match(&source) {
                OdraLocation::CratesIO(source)
            } else {
                // branch
                OdraLocation::Remote(ODRA_TEMPLATE_GH_REPO.to_string(), Some(source))
            }
        }
    }
    fn odra_latest_version() -> String {
        let response: serde_json::Value = ureq::get(ODRA_GITHUB_API_DATA)
            .call()
            .unwrap_or_else(|_| {
                Error::FailedToFetchTemplate(ODRA_GITHUB_API_DATA.to_string()).print_and_die()
            })
            .into_json()
            .unwrap_or_else(|_| {
                Error::FailedToParseTemplate(ODRA_GITHUB_API_DATA.to_string()).print_and_die()
            });
        response["tag_name"].as_str().unwrap().to_string()
    }

    fn odra_project_dependency(
        odra_location: OdraLocation,
        crate_name: &str,
        init: bool,
    ) -> Dependency {
        let (version, path, git, branch) = match odra_location {
            OdraLocation::Local(path) => {
                let path = match init {
                    true => path,
                    false => PathBuf::from("..").join(path),
                };
                let path = path
                    .join(crate_name)
                    .into_os_string()
                    .to_str()
                    .unwrap()
                    .to_string();
                (None, Some(path), None, None)
            }
            OdraLocation::Remote(repo, branch) => match branch {
                None => (Some(Self::odra_latest_version()), None, None, None),
                Some(branch) => (None, None, Some(repo), Some(branch)),
            },
            OdraLocation::CratesIO(version) => (Some(version), None, None, None),
        };

        Dependency::Detailed(DependencyDetail {
            version,
            registry: None,
            registry_index: None,
            path,
            inherited: false,
            git,
            branch,
            tag: None,
            rev: None,
            features: vec![],
            optional: false,
            default_features: false,
            package: None,
        })
    }
}
