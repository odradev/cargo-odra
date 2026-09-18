//! Module managing Odra.toml configuration.

use std::path::{Path, PathBuf};

use serde_derive::{Deserialize, Serialize};

use crate::{
    cargo_toml::load_cargo_toml,
    command,
    errors::{Error, Error::MalformedFqn},
    project::{OdraLocation, Project},
};

/// Struct describing contract.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Contract {
    pub fqn: String,
}

/// The crate a contract's struct is defined in, and how the fqn pointed at it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContractSource {
    /// A workspace member of this project; the first fqn segment is its crate name.
    Member(String),
    /// The project crate itself; the leading fqn segments are module names inside it.
    ProjectCrate(String),
    /// A crate this project depends on; the first fqn segment is its crate name.
    Dependency(String),
}

impl ContractSource {
    /// Name of the crate that defines the struct, with `-` replaced by `_`.
    pub fn crate_name(&self) -> &str {
        match self {
            ContractSource::Member(name)
            | ContractSource::ProjectCrate(name)
            | ContractSource::Dependency(name) => name,
        }
    }
}

impl Contract {
    /// First segment of the fqn. Depending on the project layout it is a workspace member's
    /// crate name, a dependency's crate name, or a module name inside the project crate.
    pub fn module_name(&self) -> String {
        self.fqn
            .split_terminator("::")
            .next()
            .unwrap_or_else(|| MalformedFqn.print_and_die())
            .replace('-', "_")
            .to_string()
    }

    /// Last segment of the fqn - the name of the module struct.
    pub fn struct_name(&self) -> String {
        self.fqn
            .split_terminator("::")
            .last()
            .unwrap_or_else(|| MalformedFqn.print_and_die())
            .to_string()
    }

    /// Classifies the contract by the first segment of its fqn.
    ///
    /// The segment names a workspace member, a crate this project depends on, or - in a single
    /// crate project - a module of the project crate.
    pub fn source(&self, project: &Project) -> ContractSource {
        let first = self.module_name();

        if project
            .members
            .iter()
            .any(|member| member.name.replace('-', "_") == first)
        {
            return ContractSource::Member(first);
        }

        if !project.is_cargo_workspace() {
            if first == project.project_crate_name() {
                return ContractSource::ProjectCrate(first);
            }
            if is_a_dependency(&project.cargo_toml_location, &first) {
                return ContractSource::Dependency(first);
            }
            // A single crate project addresses its own contracts by module path, so anything
            // else is a module of the project crate.
            return ContractSource::ProjectCrate(project.project_crate_name());
        }

        if self.dependency_hosts(project).next().is_some()
            || declared_in_workspace(&project.cargo_toml_location, &first)
        {
            return ContractSource::Dependency(first);
        }

        Error::CrateOfContractNotFound(self.fqn.clone()).print_and_die()
    }

    /// Name of the crate that defines the contract's struct.
    pub fn defining_crate(&self, project: &Project) -> String {
        self.source(project).crate_name().to_string()
    }

    /// Name of the crate whose `_build_contract` and `_build_schema` binaries build the
    /// contract.
    ///
    /// For a contract defined in the project itself this is the crate that defines it. A
    /// contract that comes from a dependency is built by the crate that depends on it: the
    /// project crate in a single crate project, the first workspace member listing the
    /// dependency in a workspace.
    pub fn host_crate_name(&self, project: &Project) -> String {
        self.host_crate(project).name
    }

    /// The crate whose binaries build the contract, and its root directory.
    pub fn host_crate(&self, project: &Project) -> HostCrate {
        let project_crate = || HostCrate {
            name: project.project_crate_name(),
            package: package_name_of(&project.cargo_toml_location)
                .unwrap_or_else(|| project.name.clone()),
            root: project.project_root(),
        };
        match self.source(project) {
            ContractSource::ProjectCrate(_) => project_crate(),
            ContractSource::Member(name) => {
                let root = project
                    .members
                    .iter()
                    .find(|member| member.name.replace('-', "_") == name)
                    .map(|member| member.root.clone())
                    .unwrap_or_else(|| project.project_root());
                let package = package_name_of(&root.join("Cargo.toml"))
                    .unwrap_or_else(|| name.replace('_', "-"));
                HostCrate {
                    name,
                    package,
                    root,
                }
            }
            ContractSource::Dependency(dependency) => {
                if !project.is_cargo_workspace() {
                    return project_crate();
                }
                self.dependency_hosts(project).next().unwrap_or_else(|| {
                    Error::NoCrateDependsOn(self.fqn.clone(), dependency.clone()).print_and_die()
                })
            }
        }
    }

    /// Value of the `ODRA_MODULE` environment variable used to build this contract.
    pub fn odra_module_value(&self, project: &Project) -> String {
        odra_module_value(
            &self.defining_crate(project),
            &self.struct_name(),
            &project.project_odra_location(),
        )
    }

    /// Workspace members that list the first fqn segment among their dependencies.
    fn dependency_hosts<'a>(
        &'a self,
        project: &'a Project,
    ) -> impl Iterator<Item = HostCrate> + 'a {
        let dependency = self.module_name();
        project
            .workspace_members
            .iter()
            .filter(move |member_root| {
                is_a_dependency(&member_root.join("Cargo.toml"), &dependency)
            })
            .filter_map(|member_root| {
                package_name_of(&member_root.join("Cargo.toml")).map(|package| HostCrate {
                    name: package.replace('-', "_"),
                    package,
                    root: member_root.clone(),
                })
            })
    }
}

/// The crate that builds a contract: the name cargo knows it by and its root directory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostCrate {
    /// Crate name with `-` replaced by `_`, the way the `_build_contract` binary is named.
    pub name: String,
    /// Package name exactly as Cargo knows it, for `--package`. `cargo build -p my_flipper`
    /// does not match a package called `my-flipper`.
    pub package: String,
    pub root: PathBuf,
}

/// Value of the `ODRA_MODULE` environment variable for a struct defined in `defining_crate`.
///
/// Odra 3.0.0 gates the wasm parts of a module on `any(odra_module = "<Struct>", odra_module =
/// "<crate>::<Struct>")`, so that two crates defining a struct of the same name do not both
/// compile their entry points into one wasm. Older Odra releases only know the bare name and
/// would build an empty wasm if given the qualified one, so they keep getting the bare name.
/// A git or local Odra is assumed to be 3.0.0 or newer, as that is what it is used for.
pub fn odra_module_value(
    defining_crate: &str,
    struct_name: &str,
    odra_location: &OdraLocation,
) -> String {
    if supports_qualified_odra_module(odra_location) {
        format!("{}::{}", defining_crate.replace('-', "_"), struct_name)
    } else {
        struct_name.to_string()
    }
}

/// Whether the Odra the project depends on understands a crate-qualified `ODRA_MODULE`.
fn supports_qualified_odra_module(odra_location: &OdraLocation) -> bool {
    match odra_location {
        OdraLocation::Local(_) | OdraLocation::Remote(_, _) => true,
        OdraLocation::CratesIO(version) => major_version(version).is_some_and(|major| major >= 3),
    }
}

/// Major version of a version requirement such as `2.9.1`, `^3.0.0` or `>=2.9, <3`.
fn major_version(version: &str) -> Option<u64> {
    version
        .trim_start_matches(|c: char| !c.is_ascii_digit())
        .split(|c: char| !c.is_ascii_digit())
        .next()
        .and_then(|major| major.parse().ok())
}

/// Package name of a manifest, with `-` replaced by `_`.
/// The `[package] name` of a manifest, exactly as written.
fn package_name_of(manifest_path: &Path) -> Option<String> {
    load_cargo_toml(manifest_path)
        .package
        .map(|package| package.name)
}

/// Checks if `crate_name` is listed in the `[dependencies]` of a manifest. Dependencies
/// inherited from a workspace (`dep = { workspace = true }`) count, as they are resolved when
/// the manifest is loaded.
fn is_a_dependency(manifest_path: &Path, crate_name: &str) -> bool {
    load_cargo_toml(manifest_path)
        .dependencies
        .keys()
        .any(|name| name.replace('-', "_") == crate_name)
}

/// Checks if `crate_name` is listed in the `[workspace.dependencies]` of a manifest.
fn declared_in_workspace(manifest_path: &Path, crate_name: &str) -> bool {
    load_cargo_toml(manifest_path)
        .workspace
        .map(|workspace| {
            workspace
                .dependencies
                .keys()
                .any(|name| name.replace('-', "_") == crate_name)
        })
        .unwrap_or(false)
}

/// Odra configuration.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct OdraToml {
    /// Contracts in the project.
    #[serde(default)]
    pub contracts: Vec<Contract>,
    #[serde(skip)]
    pub location: PathBuf,
}

impl OdraToml {
    /// Loads configuration from Odra.toml file.
    pub fn load(location: &Path) -> OdraToml {
        let odra_conf = command::read_file_content(location.to_path_buf());
        let mut odra_toml: OdraToml = match odra_conf {
            Ok(conf_file) => toml::from_str(conf_file.as_str()).unwrap(),
            Err(_) => Error::OdraTomlNotFound(location.to_path_buf()).print_and_die(),
        };

        odra_toml.location = location.to_path_buf();
        odra_toml
    }

    /// Saves configuration into Odra.toml file.
    pub fn save(&self) -> Result<(), Error> {
        let content = toml::to_string(&self)?;
        command::write_to_file(self.location.clone(), &content)
    }

    /// Check if the contract is defined in Odra.toml file.
    pub fn has_contract(&self, contract_name: &str) -> bool {
        self.contracts
            .iter()
            .any(|c| c.struct_name() == contract_name)
    }

    /// Check if any contract in Odra.toml is a part of a crate with given name
    pub fn crate_has_contracts(&self, crate_name: &str) -> bool {
        self.contracts.iter().any(|c| {
            c.fqn
                .split_terminator("::")
                .next()
                .unwrap_or_else(|| Error::MalformedFqn.print_and_die())
                == crate_name.replace('-', "_")
        })
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        sync::atomic::{AtomicUsize, Ordering},
    };

    use super::*;
    use crate::project::Member;

    static COUNTER: AtomicUsize = AtomicUsize::new(0);

    /// A fresh directory under the system temp dir, removed on drop.
    struct TempDir(PathBuf);

    impl TempDir {
        fn new() -> Self {
            let id = COUNTER.fetch_add(1, Ordering::SeqCst);
            let path = std::env::temp_dir()
                .join(format!("cargo-odra-toml-test-{}-{id}", std::process::id()));
            fs::create_dir_all(&path).unwrap();
            Self(path)
        }

        fn write(&self, relative: &str, content: &str) -> PathBuf {
            let path = self.0.join(relative);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(&path, content).unwrap();
            path
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn contract(fqn: &str) -> Contract {
        Contract {
            fqn: fqn.to_string(),
        }
    }

    /// A single crate project depending on `odra-modules`.
    fn single_crate_project(dir: &TempDir) -> Project {
        let cargo_toml = dir.write(
            "Cargo.toml",
            r#"
[package]
name = "my-project"
version = "0.1.0"

[dependencies]
odra = "3.0.0"
odra-modules = "3.0.0"
"#,
        );
        Project {
            name: "my-project".to_string(),
            project_root: dir.0.clone(),
            cargo_toml_location: cargo_toml,
            odra_toml_location: dir.0.join("Odra.toml"),
            members: vec![],
            workspace_members: vec![],
        }
    }

    /// A workspace with a `flipper` member that depends on `odra-modules` and a `cli` member
    /// that does not.
    fn workspace_project(dir: &TempDir) -> Project {
        let cargo_toml = dir.write(
            "Cargo.toml",
            r#"
[workspace]
members = ["flipper", "cli"]

[workspace.dependencies]
odra = "3.0.0"
"#,
        );
        dir.write(
            "flipper/Cargo.toml",
            r#"
[package]
name = "flipper"
version = "0.1.0"

[dependencies]
odra = { workspace = true }
odra-modules = "3.0.0"
"#,
        );
        dir.write(
            "cli/Cargo.toml",
            r#"
[package]
name = "cli"
version = "0.1.0"

[dependencies]
odra = { workspace = true }
"#,
        );
        Project {
            name: "workspace".to_string(),
            project_root: dir.0.clone(),
            cargo_toml_location: cargo_toml,
            odra_toml_location: dir.0.join("Odra.toml"),
            members: vec![Member {
                name: "flipper".to_string(),
                root: dir.0.join("flipper"),
            }],
            // `cli` first, to prove the host is picked by its dependencies, not by order.
            workspace_members: vec![dir.0.join("cli"), dir.0.join("flipper")],
        }
    }

    #[test]
    fn reads_the_first_and_the_last_fqn_segment() {
        assert_eq!(
            contract("odra_modules::erc20::Erc20").module_name(),
            "odra_modules"
        );
        assert_eq!(
            contract("odra_modules::erc20::Erc20").struct_name(),
            "Erc20"
        );
        assert_eq!(contract("flipper::Flipper").module_name(), "flipper");
        assert_eq!(contract("flipper::Flipper").struct_name(), "Flipper");
        // A dash in a crate name is what cargo makes of it.
        assert_eq!(
            contract("odra-modules::Erc20").module_name(),
            "odra_modules"
        );
    }

    #[test]
    fn a_module_of_a_single_crate_project_is_built_by_that_crate() {
        let dir = TempDir::new();
        let project = single_crate_project(&dir);
        let contract = contract("features::events::PartyContract");

        assert_eq!(
            contract.source(&project),
            ContractSource::ProjectCrate("my_project".to_string())
        );
        assert_eq!(contract.defining_crate(&project), "my_project");
        assert_eq!(contract.host_crate_name(&project), "my_project");
    }

    #[test]
    fn the_project_crate_may_be_named_explicitly() {
        let dir = TempDir::new();
        let project = single_crate_project(&dir);
        let contract = contract("my_project::features::PartyContract");

        assert_eq!(
            contract.source(&project),
            ContractSource::ProjectCrate("my_project".to_string())
        );
    }

    #[test]
    fn a_dependency_of_a_single_crate_project_is_built_by_the_project_crate() {
        let dir = TempDir::new();
        let project = single_crate_project(&dir);
        let contract = contract("odra_modules::erc20::Erc20");

        assert_eq!(
            contract.source(&project),
            ContractSource::Dependency("odra_modules".to_string())
        );
        assert_eq!(contract.defining_crate(&project), "odra_modules");
        assert_eq!(contract.host_crate_name(&project), "my_project");
    }

    #[test]
    fn a_workspace_member_builds_its_own_contracts() {
        let dir = TempDir::new();
        let project = workspace_project(&dir);
        let contract = contract("flipper::Flipper");

        assert_eq!(
            contract.source(&project),
            ContractSource::Member("flipper".to_string())
        );
        assert_eq!(contract.defining_crate(&project), "flipper");
        assert_eq!(contract.host_crate_name(&project), "flipper");
    }

    #[test]
    fn a_dashed_member_is_asked_for_by_its_real_package_name() {
        // `cargo build -p my_flipper` does not match a package called `my-flipper`, while the
        // `_build_contract` binary of that package is named with underscores.
        let dir = TempDir::new();
        let cargo_toml = dir.write(
            "Cargo.toml",
            r#"
[workspace]
members = ["my-flipper"]

[workspace.dependencies]
odra = "3.0.0"
"#,
        );
        dir.write(
            "my-flipper/Cargo.toml",
            r#"
[package]
name = "my-flipper"
version = "0.1.0"

[dependencies]
odra = { workspace = true }
odra-modules = "3.0.0"
"#,
        );
        let project = Project {
            name: "workspace".to_string(),
            project_root: dir.0.clone(),
            cargo_toml_location: cargo_toml,
            odra_toml_location: dir.0.join("Odra.toml"),
            members: vec![Member {
                name: "my-flipper".to_string(),
                root: dir.0.join("my-flipper"),
            }],
            workspace_members: vec![dir.0.join("my-flipper")],
        };

        let own = contract("my_flipper::Flipper").host_crate(&project);
        assert_eq!(own.name, "my_flipper");
        assert_eq!(own.package, "my-flipper");

        let external = contract("odra_modules::erc20::Erc20").host_crate(&project);
        assert_eq!(external.name, "my_flipper");
        assert_eq!(external.package, "my-flipper");
    }

    #[test]
    fn a_dependency_of_a_workspace_is_built_by_the_member_that_depends_on_it() {
        let dir = TempDir::new();
        let project = workspace_project(&dir);
        let contract = contract("odra_modules::erc20::Erc20");

        assert_eq!(
            contract.source(&project),
            ContractSource::Dependency("odra_modules".to_string())
        );
        assert_eq!(contract.defining_crate(&project), "odra_modules");
        assert_eq!(contract.host_crate_name(&project), "flipper");
        assert_eq!(contract.host_crate(&project).root, dir.0.join("flipper"));
    }

    #[test]
    fn a_workspace_with_only_external_contracts_still_finds_the_host() {
        let dir = TempDir::new();
        let mut project = workspace_project(&dir);
        // No member owns a contract, so `members` is empty - the project is still a workspace.
        project.members = vec![];
        let contract = contract("odra_modules::erc20::Erc20");

        assert!(project.is_cargo_workspace());
        assert_eq!(contract.host_crate_name(&project), "flipper");
    }

    #[test]
    fn qualifies_the_odra_module_for_odra_3_and_newer() {
        let cases = [
            (
                OdraLocation::CratesIO("3.0.0".to_string()),
                "odra_modules::Erc20",
            ),
            (
                OdraLocation::CratesIO("^3.1".to_string()),
                "odra_modules::Erc20",
            ),
            (
                OdraLocation::CratesIO("4".to_string()),
                "odra_modules::Erc20",
            ),
            (
                OdraLocation::Local(PathBuf::from("/odra")),
                "odra_modules::Erc20",
            ),
            (
                OdraLocation::Remote("https://github.com/odradev/odra".to_string(), None),
                "odra_modules::Erc20",
            ),
            (OdraLocation::CratesIO("2.9.1".to_string()), "Erc20"),
            (OdraLocation::CratesIO("2.9".to_string()), "Erc20"),
            (OdraLocation::CratesIO("1.0.0".to_string()), "Erc20"),
            // A range cannot be trusted to be 3.0.0 or newer.
            (OdraLocation::CratesIO(">=2.9, <4".to_string()), "Erc20"),
        ];
        for (location, expected) in cases {
            assert_eq!(
                odra_module_value("odra-modules", "Erc20", &location),
                expected,
                "location: {location:?}"
            );
        }
    }
}
