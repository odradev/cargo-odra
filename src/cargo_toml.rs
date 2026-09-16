//! Module containing functions used by Builder for managing its Cargo.toml file
use std::path::{Path, PathBuf};

use cargo_toml::{Dependency, DependencyDetail, Manifest};

use crate::{command, errors::Error, project::OdraLocation, utils::odra_latest_version};

/// Returns Cargo.toml as Manifest struct.
///
/// Fields inherited from a workspace (`key = { workspace = true }`) are resolved the way
/// Cargo does it, so a contract crate that lives inside a bigger workspace loads correctly.
pub fn load_cargo_toml(path: &Path) -> Manifest {
    match read_manifest(path) {
        Ok(manifest) => manifest,
        Err(err) => {
            Error::FailedToReadCargo(err).print_and_die();
        }
    }
}

fn read_manifest(path: &Path) -> Result<Manifest, String> {
    let path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let mut manifest = parse_manifest(&path)?;
    let workspace = find_workspace(&path, &manifest)?;
    manifest
        .complete_from_path_and_workspace(
            &path,
            workspace
                .as_ref()
                .map(|(ws, ws_path)| (ws, ws_path.as_path())),
        )
        .map_err(|err| err.to_string())?;
    Ok(manifest)
}

fn parse_manifest(path: &Path) -> Result<Manifest, String> {
    let content = std::fs::read(path).map_err(|err| format!("{}: {err}", path.display()))?;
    Manifest::from_slice(&content).map_err(|err| format!("{}: {err}", path.display()))
}

/// Finds the workspace a package manifest belongs to, following Cargo's rules:
/// the `package.workspace` key wins, otherwise it is the nearest ancestor `Cargo.toml`
/// with a `[workspace]` section. Ancestor manifests without one (e.g. a crate that merely
/// contains the package's directory) are skipped, which is what `cargo_toml` itself
/// does not do.
///
/// Returns `None` if the manifest is its own workspace root or no workspace was found;
/// `cargo_toml` then behaves as before.
fn find_workspace(
    manifest_path: &Path,
    manifest: &Manifest,
) -> Result<Option<(Manifest, PathBuf)>, String> {
    if manifest.workspace.is_some() {
        return Ok(None);
    }
    let Some(manifest_dir) = manifest_path.parent() else {
        return Ok(None);
    };

    if let Some(hint) = manifest
        .package
        .as_ref()
        .and_then(|p| p.workspace.as_deref())
    {
        let workspace_path = manifest_dir.join(hint).join("Cargo.toml");
        let workspace = parse_manifest(&workspace_path)?;
        return Ok(Some((workspace, workspace_path)));
    }

    for ancestor in manifest_dir.ancestors().skip(1) {
        let candidate = ancestor.join("Cargo.toml");
        if !candidate.is_file() {
            continue;
        }
        let candidate_manifest = parse_manifest(&candidate)?;
        if candidate_manifest.workspace.is_some() {
            return Ok(Some((candidate_manifest, candidate)));
        }
    }
    Ok(None)
}

/// Saves configuration into Odra.toml file.
pub fn save_cargo_toml(path: PathBuf, cargo_toml: &Manifest) -> Result<(), Error> {
    let content = toml::to_string(cargo_toml)?;
    command::write_to_file(path, &content)
}

pub fn odra_project_dependency(
    odra_location: &OdraLocation,
    crate_path: &str,
    optional: bool,
    init: bool,
) -> Dependency {
    let (version, path, git, branch) = match odra_location {
        OdraLocation::Local(path) => {
            let path = match init {
                true => path.clone(),
                false => PathBuf::from("..").join(path),
            };
            let path = path
                .join(crate_path)
                .into_os_string()
                .to_str()
                .unwrap()
                .to_string();
            (None, Some(path), None, None)
        }
        OdraLocation::Remote(repo, branch) => match branch {
            None => (Some(odra_latest_version()), None, None, None),
            Some(branch) => (None, None, Some(repo), Some(branch)),
        },
        OdraLocation::CratesIO(version) => (Some(version.clone()), None, None, None),
    };

    Dependency::Detailed(DependencyDetail {
        version,
        registry: None,
        registry_index: None,
        path,
        inherited: false,
        git: git.cloned(),
        branch: branch.cloned(),
        tag: None,
        rev: None,
        features: vec![],
        optional,
        default_features: false,
        package: None,
    })
}

pub fn odra_project_dependency_string(
    odra_location: &OdraLocation,
    crate_path: &str,
    init: bool,
) -> String {
    toml::to_string(&odra_project_dependency(
        odra_location,
        crate_path,
        false,
        init,
    ))
    .expect("Failed to serialize odra dependency.")
    .trim_end()
    .replace('\n', ", ")
}

pub fn opt_odra_project_dependency_string(
    odra_location: &OdraLocation,
    crate_path: &str,
    init: bool,
) -> String {
    toml::to_string(&odra_project_dependency(
        odra_location,
        crate_path,
        true,
        init,
    ))
    .expect("Failed to serialize odra dependency.")
    .trim_end()
    .replace('\n', ", ")
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        sync::atomic::{AtomicUsize, Ordering},
    };

    use super::*;

    static COUNTER: AtomicUsize = AtomicUsize::new(0);

    /// A fresh directory under the system temp dir, removed on drop.
    struct TempDir(PathBuf);

    impl TempDir {
        fn new() -> Self {
            let id = COUNTER.fetch_add(1, Ordering::SeqCst);
            let path =
                std::env::temp_dir().join(format!("cargo-odra-test-{}-{id}", std::process::id()));
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

    const WORKSPACE: &str = r#"
[workspace]
members = ["examples/pkg"]

[workspace.package]
license = "MIT"
"#;

    const PLAIN_PACKAGE: &str = r#"
[package]
name = "examples"
version = "0.1.0"
"#;

    const INHERITING_PACKAGE: &str = r#"
[package]
name = "pkg"
version = "0.1.0"
license = { workspace = true }
"#;

    #[test]
    fn inherits_through_a_non_workspace_parent() {
        let dir = TempDir::new();
        dir.write("Cargo.toml", WORKSPACE);
        dir.write("examples/Cargo.toml", PLAIN_PACKAGE);
        let path = dir.write("examples/pkg/Cargo.toml", INHERITING_PACKAGE);

        let manifest = read_manifest(&path).unwrap();
        assert_eq!(manifest.package.unwrap().license(), Some("MIT"));
    }

    #[test]
    fn honours_explicit_package_workspace() {
        let dir = TempDir::new();
        dir.write("ws/Cargo.toml", WORKSPACE);
        let path = dir.write(
            "elsewhere/pkg/Cargo.toml",
            r#"
[package]
name = "pkg"
version = "0.1.0"
workspace = "../../ws"
license = { workspace = true }
"#,
        );

        let manifest = read_manifest(&path).unwrap();
        assert_eq!(manifest.package.unwrap().license(), Some("MIT"));
    }

    #[test]
    fn standalone_package_loads_as_before() {
        let dir = TempDir::new();
        let path = dir.write(
            "pkg/Cargo.toml",
            r#"
[package]
name = "pkg"
version = "0.1.0"
license = "Apache-2.0"
"#,
        );

        let manifest = read_manifest(&path).unwrap();
        let package = manifest.package.unwrap();
        assert_eq!(package.name, "pkg");
        assert_eq!(package.license(), Some("Apache-2.0"));
    }

    #[test]
    fn workspace_root_loads_its_members() {
        let dir = TempDir::new();
        let path = dir.write("Cargo.toml", WORKSPACE);

        let manifest = read_manifest(&path).unwrap();
        assert_eq!(manifest.workspace.unwrap().members, vec!["examples/pkg"]);
    }

    #[test]
    fn reports_missing_workspace_field() {
        let dir = TempDir::new();
        dir.write("Cargo.toml", "[workspace]\nmembers = [\"pkg\"]\n");
        let path = dir.write("pkg/Cargo.toml", INHERITING_PACKAGE);

        let err = read_manifest(&path).unwrap_err();
        assert!(err.contains("workspace.package"), "{err}");
    }
}
