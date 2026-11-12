//! Skipping the build of a contract whose wasm is up to date.
//!
//! Cargo cannot tell on its own: every contract is built from the same crate with a different
//! `odra_module` cfg, so to Cargo the crate is dirty at every build. Cargo does leave a dep-info
//! file (`<bin>.d`) next to the wasm it built, listing every file of the project the wasm was
//! compiled from, build script and `rerun-if-changed` files included. Dependencies from the
//! registry are not listed; their versions are fixed by `Cargo.lock`, which is hashed instead,
//! as are the manifests and the toolchain.
//!
//! A fingerprint is written per contract after a build; the build of a contract whose
//! fingerprint still matches is skipped. A fingerprint that cannot be written or read only
//! costs a rebuild, never the build itself.

use std::{
    collections::BTreeMap,
    fs,
    hash::{Hash, Hasher},
    io,
    path::{Path, PathBuf},
};

use serde_derive::{Deserialize, Serialize};

use crate::{command, log};

/// Bumped when the format or the meaning of a fingerprint changes, so older ones never match.
const VERSION: u32 = 1;

/// Environment variables Cargo and rustc read that change what a build produces.
const ENV_INPUTS: &[&str] = &[
    "RUSTFLAGS",
    "CARGO_ENCODED_RUSTFLAGS",
    "CARGO_BUILD_RUSTFLAGS",
    "CARGO_BUILD_TARGET",
    "RUSTC",
    "RUSTC_WRAPPER",
    "RUSTC_WORKSPACE_WRAPPER",
    "RUSTUP_TOOLCHAIN",
];

/// What a contract's wasm was built from, apart from the files.
///
/// Built once per `cargo odra` run; `inputs` adds what differs per contract.
pub struct Environment {
    common: BTreeMap<String, String>,
}

impl Environment {
    /// Probes the toolchain. `wasm_opt` describes the wasm-opt the wasm files are passed through.
    pub fn detect(wasm_opt: &str) -> Self {
        let mut common = BTreeMap::new();
        common.insert(
            "cargo-odra".to_string(),
            env!("CARGO_PKG_VERSION").to_string(),
        );
        common.insert(
            "rustc".to_string(),
            command::command_output("rustc --version --verbose"),
        );
        common.insert("wasm-opt".to_string(), wasm_opt.to_string());
        for (name, value) in std::env::vars() {
            if ENV_INPUTS.contains(&name.as_str()) || name.starts_with("CARGO_PROFILE_") {
                common.insert(format!("env:{name}"), value);
            }
        }
        Environment { common }
    }

    /// The environment of one contract's build: the common part plus `details`, such as the
    /// `ODRA_MODULE` value and the package the wasm is built from.
    pub fn inputs(&self, details: &[(&str, &str)]) -> BTreeMap<String, String> {
        let mut inputs = self.common.clone();
        for (name, value) in details {
            inputs.insert(name.to_string(), value.to_string());
        }
        inputs
    }
}

/// The fingerprint of one contract's wasm, as written to disk.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
struct Fingerprint {
    version: u32,
    /// Toolchain, flags, module: see [`Environment`].
    environment: BTreeMap<String, String>,
    /// A content hash per file the wasm was built from, `None` for a path that did not exist
    /// (Cargo lists `rerun-if-changed` files that are yet to be written).
    files: BTreeMap<String, Option<String>>,
}

/// Where the fingerprint of `contract` of the project at `project_root` is kept: under the
/// project's target directory, so `cargo clean` removes it along with the wasm's build.
///
/// Projects that share a target directory, like the members of a workspace, may list a
/// contract of the same name (one from a dependency crate, say), each with its own wasm and
/// its own inputs, so the fingerprints are kept apart by project.
pub fn path(target_dir: &Path, project_root: &Path, contract: &str) -> PathBuf {
    let mut hasher = std::hash::DefaultHasher::new();
    project_root.hash(&mut hasher);
    let project = format!(
        "{}-{:016x}",
        project_root.file_name().map_or_else(
            || "project".to_string(),
            |name| name.to_string_lossy().into_owned()
        ),
        hasher.finish()
    );
    target_dir
        .join("cargo-odra")
        .join("fingerprints")
        .join(project)
        .join(format!("{contract}.json"))
}

/// Whether the wasm at `wasm` was built from exactly the files and environment recorded at
/// `fingerprint`. Anything unreadable or missing counts as not fresh.
pub fn is_fresh(fingerprint: &Path, wasm: &Path, environment: &BTreeMap<String, String>) -> bool {
    if !wasm.is_file() {
        return false;
    }
    let Ok(content) = fs::read_to_string(fingerprint) else {
        return false;
    };
    let Ok(recorded) = serde_json::from_str::<Fingerprint>(&content) else {
        return false;
    };
    recorded.version == VERSION
        && recorded.environment == *environment
        && recorded
            .files
            .iter()
            .all(|(file, hash)| hash_file(Path::new(file)).ok().as_ref() == Some(hash))
}

/// Writes the fingerprint of a wasm just built: the files Cargo's dep-info at `dep_info` lists,
/// plus `extra_files` (the manifests, the lock file, the wasm itself), under `environment`.
pub fn record(
    fingerprint: &Path,
    dep_info: &Path,
    extra_files: &[PathBuf],
    environment: BTreeMap<String, String>,
) -> io::Result<()> {
    let dep_info_content = fs::read_to_string(dep_info)?;
    let mut files = BTreeMap::new();
    for file in dep_info_files(&dep_info_content)
        .iter()
        .chain(extra_files.iter())
    {
        files.insert(file.display().to_string(), hash_file(file)?);
    }
    let recorded = Fingerprint {
        version: VERSION,
        environment,
        files,
    };
    if let Some(dir) = fingerprint.parent() {
        fs::create_dir_all(dir)?;
    }
    fs::write(fingerprint, serde_json::to_string(&recorded)?)
}

/// Writes the fingerprint and, when that fails, says so and goes on: a missing fingerprint
/// costs a rebuild next time, nothing more.
pub fn record_or_warn(
    fingerprint: &Path,
    dep_info: &Path,
    extra_files: &[PathBuf],
    environment: BTreeMap<String, String>,
) {
    if let Err(error) = record(fingerprint, dep_info, extra_files, environment) {
        log::warn(format!(
            "Could not record what {} was built from ({error}); it will be rebuilt next time.",
            fingerprint.display()
        ));
    }
}

/// The content hash of a file, or `None` if there is no file at `path`.
fn hash_file(path: &Path) -> io::Result<Option<String>> {
    match fs::read(path) {
        Ok(content) => {
            let mut hasher = std::hash::DefaultHasher::new();
            content.hash(&mut hasher);
            Ok(Some(format!("{:016x}", hasher.finish())))
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error),
    }
}

/// The files a Makefile-style dep-info lists: every line is `target: file file ...`, with a
/// space in a file name escaped as `\ `.
fn dep_info_files(dep_info: &str) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for line in dep_info.lines() {
        let Some((_, dependencies)) = line.split_once(": ") else {
            continue;
        };
        let mut current = String::new();
        let mut chars = dependencies.chars().peekable();
        while let Some(c) = chars.next() {
            match c {
                '\\' if chars.peek() == Some(&' ') => {
                    current.push(' ');
                    chars.next();
                }
                ' ' => {
                    if !current.is_empty() {
                        files.push(PathBuf::from(std::mem::take(&mut current)));
                    }
                }
                _ => current.push(c),
            }
        }
        if !current.is_empty() {
            files.push(PathBuf::from(current));
        }
    }
    files.sort();
    files.dedup();
    files
}

#[cfg(test)]
mod tests {
    use std::{collections::BTreeMap, fs, path::PathBuf};

    use super::{dep_info_files, is_fresh, path, record};

    #[test]
    fn fingerprints_are_kept_apart_by_project() {
        let target = std::path::Path::new("/ws/target");
        let modules = path(target, std::path::Path::new("/ws/modules"), "Erc20");
        let examples = path(target, std::path::Path::new("/ws/examples"), "Erc20");
        assert_ne!(modules, examples);
        assert!(modules.starts_with("/ws/target/cargo-odra/fingerprints"));
        assert!(modules
            .parent()
            .and_then(|dir| dir.file_name())
            .is_some_and(|dir| dir.to_string_lossy().starts_with("modules-")));
        assert_eq!(modules.file_name().unwrap(), "Erc20.json");
        assert_eq!(
            modules,
            path(target, std::path::Path::new("/ws/modules"), "Erc20")
        );
    }

    #[test]
    fn reads_the_files_out_of_a_dep_info() {
        let dep_info = "/t/release/flipper.wasm: /p/src/lib.rs /p/build.rs /p/my\\ dir/a.json\n\
                        /t/release/flipper.d: /p/src/lib.rs\n";
        assert_eq!(
            dep_info_files(dep_info),
            vec![
                PathBuf::from("/p/build.rs"),
                PathBuf::from("/p/my dir/a.json"),
                PathBuf::from("/p/src/lib.rs"),
            ]
        );
    }

    fn environment(module: &str) -> BTreeMap<String, String> {
        BTreeMap::from([("odra-module".to_string(), module.to_string())])
    }

    #[test]
    fn a_wasm_is_fresh_until_an_input_changes() {
        let root = std::env::temp_dir().join(format!("cargo-odra-fp-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("src")).unwrap();
        let lib = root.join("src/lib.rs");
        let lock = root.join("Cargo.lock");
        let schema = root.join("flipper_schema.json"); // listed by cargo, not written yet
        let wasm = root.join("Flipper.wasm");
        let dep_info = root.join("flipper.d");
        let fingerprint = root.join("fp/Flipper.json");
        fs::write(&lib, "pub fn flip() {}").unwrap();
        fs::write(&lock, "lock v1").unwrap();
        fs::write(&wasm, b"\0asm").unwrap();
        fs::write(
            &dep_info,
            format!(
                "{}: {} {}\n",
                root.join("flipper.wasm").display(),
                lib.display(),
                schema.display()
            ),
        )
        .unwrap();

        // Nothing recorded yet.
        assert!(!is_fresh(&fingerprint, &wasm, &environment("Flipper")));

        record(
            &fingerprint,
            &dep_info,
            &[lock.clone(), wasm.clone()],
            environment("Flipper"),
        )
        .unwrap();
        assert!(is_fresh(&fingerprint, &wasm, &environment("Flipper")));
        // The same content again is still fresh, whatever the mtime.
        fs::write(&lib, "pub fn flip() {}").unwrap();
        assert!(is_fresh(&fingerprint, &wasm, &environment("Flipper")));

        // A different environment, module or toolchain, is not.
        assert!(!is_fresh(&fingerprint, &wasm, &environment("Flapper")));
        // Nor a changed source file...
        fs::write(&lib, "pub fn flop() {}").unwrap();
        assert!(!is_fresh(&fingerprint, &wasm, &environment("Flipper")));
        fs::write(&lib, "pub fn flip() {}").unwrap();
        // ...a changed lock file...
        fs::write(&lock, "lock v2").unwrap();
        assert!(!is_fresh(&fingerprint, &wasm, &environment("Flipper")));
        fs::write(&lock, "lock v1").unwrap();
        // ...a file that did not exist at the build and does now...
        fs::write(&schema, "{}").unwrap();
        assert!(!is_fresh(&fingerprint, &wasm, &environment("Flipper")));
        fs::remove_file(&schema).unwrap();
        // ...a replaced wasm...
        fs::write(&wasm, b"\0asm2").unwrap();
        assert!(!is_fresh(&fingerprint, &wasm, &environment("Flipper")));
        fs::write(&wasm, b"\0asm").unwrap();
        assert!(is_fresh(&fingerprint, &wasm, &environment("Flipper")));
        // ...or a wasm that is gone.
        fs::remove_file(&wasm).unwrap();
        assert!(!is_fresh(&fingerprint, &wasm, &environment("Flipper")));

        let _ = fs::remove_dir_all(&root);
    }
}
