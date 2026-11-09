//! Module containing code that runs external commands.

use std::{
    env,
    fs::{self, File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    process::Command,
};

use clap::Parser;
use Error::InvalidInternalCommand;

use crate::{
    actions::test::TestFilters,
    cli::Cargo,
    consts::{ODRA_BACKEND_ENV_KEY, ODRA_MODULE_ENV_KEY},
    errors::Error,
    log,
    odra_toml::HostCrate,
    paths,
};

/// Returns output of a command as a String.
pub fn command_output(command: &str) -> String {
    let mut split_command: Vec<&str> = command.split(' ').collect();
    let program = *split_command
        .first()
        .unwrap_or_else(|| InvalidInternalCommand(command.to_string()).print_and_die());
    let args: Vec<&str> = split_command.drain(1..).collect();
    let output = Command::new(program).args(args).output().unwrap();
    std::str::from_utf8(output.stdout.as_slice())
        .unwrap()
        .to_string()
}

/// Copies a file.
pub fn copy_file(source: &Path, target: &Path) -> Result<(), Error> {
    fs::copy(source, target).map(|_| ()).map_err(|error| {
        Error::CommandFailed(format!(
            "Couldn't copy {} to {}: {error}",
            source.display(),
            target.display()
        ))
    })
}

/// Remove a directory.
/// Removes a file, if it is there.
pub fn rm_file(path: PathBuf) {
    if rm_rf::ensure_removed(path.clone()).is_err() {
        Error::RemoveDirNotPossible(path).print_and_die();
    }
}

pub fn rm_dir(path: PathBuf) {
    log::info(format!("Removing {}...", path.display()));
    let result = rm_rf::ensure_removed(path.clone());
    if result.is_err() {
        Error::RemoveDirNotPossible(path).print_and_die();
    };
}

/// Creates a directory.
pub fn mkdir(path: PathBuf) -> Result<(), Error> {
    fs::create_dir_all(path).map_err(Error::IO)?;
    Ok(())
}

/// Returns true if the active Rust toolchain was built on or after 2025-02-17,
/// which corresponds to nightly-2025-02-18 — the first nightly that includes the
/// LLVM 20 upgrade (rust-lang/rust#135763). LLVM 20 enables `bulk-memory` by default
/// for wasm32 targets, causing the compiler to emit `memory.copy` / `memory.fill`
/// instructions. Older LLVM versions emitted inline loops or libc-style calls instead.
///
/// The first stable release with this change is Rust 1.87.0 (2025-05-15).
///
/// Sources:
/// - https://github.com/rust-lang/rust/issues/137315 (bisected to nightly-2025-02-18)
/// - https://github.com/rust-lang/rust/pull/135763 (LLVM 20 upgrade, merged 2025-02-17)
fn needs_bulk_memory_flags() -> bool {
    let version = command_output("rustc --version");
    // Format: "rustc X.Y.Z(-nightly)? (hash YYYY-MM-DD)"
    version
        .rsplit_once(' ')
        .and_then(|(_, date)| date.trim().trim_end_matches(')').parse::<String>().ok())
        .is_some_and(|date| date.as_str() >= "2025-02-17")
}

/// Oldest binaryen release that understands `--llvm-memory-copy-fill-lowering`.
/// Older releases reject the flag outright, which makes the build fail for a reason
/// that has nothing to do with the contract being built.
const MIN_BINARYEN_VERSION: u32 = 121;

/// What a local `wasm-opt --version` tells us.
enum WasmOpt {
    /// `wasm-opt` could not be executed - not installed, or not in PATH.
    Missing,
    /// `wasm-opt` ran; holds the binaryen version if it could be read from the output.
    Found(Option<u32>),
}

/// Runs `wasm-opt --version` and reads the binaryen version out of it.
fn probe_wasm_opt() -> WasmOpt {
    match Command::new("wasm-opt").arg("--version").output() {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => WasmOpt::Missing,
        // Anything else - a permission problem, say - is not ours to diagnose.
        Err(_) => WasmOpt::Found(None),
        Ok(output) => {
            // Output looks like: "wasm-opt version 121 (version_121)"
            let version = std::str::from_utf8(&output.stdout)
                .ok()
                .and_then(|text| text.split_whitespace().nth(2)?.parse().ok());
            WasmOpt::Found(version)
        }
    }
}

/// Quits with an actionable message if `wasm-opt` is missing, or too old for the
/// flags we are about to pass it.
fn check_wasm_opt(needs_bulk_memory: bool) {
    match probe_wasm_opt() {
        WasmOpt::Missing => Error::WasmoptNotInstalled.print_and_die(),
        WasmOpt::Found(Some(version)) if needs_bulk_memory && version < MIN_BINARYEN_VERSION => {
            Error::WasmoptTooOld(version, MIN_BINARYEN_VERSION).print_and_die()
        }
        // An unreadable version is not worth failing over - let the build speak for itself.
        WasmOpt::Found(_) => {}
    }
}

/// The flags `process_wasm` passes to wasm-opt, which depend on the toolchain.
///
/// Detected once, up front: a missing or too old wasm-opt is then reported before the first
/// contract is compiled rather than after the last, and the probes do not run once per contract.
pub struct WasmOptFlags {
    needs_bulk_memory: bool,
}

impl WasmOptFlags {
    /// Probes the toolchain and wasm-opt; quits if wasm-opt is missing or too old.
    pub fn detect() -> Self {
        let needs_bulk_memory = needs_bulk_memory_flags();
        check_wasm_opt(needs_bulk_memory);
        WasmOptFlags { needs_bulk_memory }
    }
}

/// Runs wasm-opt and wasm-strip on a given contract's wasm file.
pub fn process_wasm(
    contract_name: &str,
    project_root: &Path,
    flags: &WasmOptFlags,
) -> Result<(), Error> {
    let wasm = paths::wasm_path_in_wasm_dir(contract_name, project_root);

    let mut cmd = Command::new("wasm-opt");
    cmd.current_dir(project_root).arg("--signext-lowering");
    if flags.needs_bulk_memory {
        cmd.arg("--enable-bulk-memory")
            .arg("--llvm-memory-copy-fill-lowering");
    }
    match cmd.arg(&wasm).arg("-o").arg(&wasm).status() {
        Ok(status) if status.success() => {}
        _ => return Err(Error::WasmoptDidNotFinish),
    }

    let command = Command::new("wasm-strip")
        .current_dir(project_root)
        .arg(&wasm)
        .status();
    match command {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            Err(Error::WasmstripNotInstalled)
        }
        Err(_) => Err(Error::WasmstripDidNotFinish),
        Ok(status) if !status.success() => Err(Error::WasmstripDidNotFinish),
        Ok(_) => Ok(()),
    }
}

/// Runs cargo with given args.
fn cargo(current_dir: PathBuf, command: &str, tail_args: Vec<&str>) {
    cargo_checked(&current_dir, command, tail_args, &[], false)
        .unwrap_or_else(|error| error.print_and_die());
}

/// Runs cargo with the given args and extra environment, and reports a failure instead of
/// quitting, so a build running on a worker thread can hand it back.
///
/// With `capture_output`, what cargo prints is held back and written out in one piece once it
/// has finished, so the output of builds running side by side does not interleave. Cargo is
/// quietened then, unless a verbosity was asked for: its progress lines are meant for a
/// terminal it has to itself, and the warnings and errors still come through.
fn cargo_checked(
    current_dir: &Path,
    command: &str,
    tail_args: Vec<&str>,
    env: &[(&str, &str)],
    capture_output: bool,
) -> Result<(), Error> {
    let mut args = vec![command];
    match verbosity_arg() {
        Some(verbosity) => args.push(verbosity),
        None if capture_output => args.push("--quiet"),
        None => {}
    }
    args.extend(tail_args);

    let mut cargo = Command::new("cargo");
    cargo
        .current_dir(current_dir)
        .args(args.as_slice())
        .envs(env.iter().copied());
    let status = if capture_output {
        let output = cargo.output()?;
        let mut stderr = std::io::stderr().lock();
        stderr.write_all(&output.stdout)?;
        stderr.write_all(&output.stderr)?;
        output.status
    } else {
        cargo.status()?
    };

    if status.success() {
        Ok(())
    } else {
        Err(Error::CommandFailed(format!(
            "Couldn't run cargo with args {args:?}"
        )))
    }
}

/// Builds the wasm file of one contract.
///
/// `odra_module` is the value of the `ODRA_MODULE` environment variable the contract is gated
/// on, `host_crate` the crate whose binaries build it: its `name` names the binary and its
/// `package` is what Cargo is asked for with `--package`. The build happens in `target_dir`
/// when given, in the project's target directory otherwise; see `cargo_checked` for
/// `capture_output`.
pub fn cargo_build_wasm_files(
    current_dir: &Path,
    odra_module: &str,
    host_crate: &HostCrate,
    is_workspace: bool,
    target_dir: Option<&Path>,
    capture_output: bool,
) -> Result<(), Error> {
    let build_contract = format!("{}_build_contract", host_crate.name);
    let mut params = vec![
        "--target",
        "wasm32-unknown-unknown",
        "--bin",
        &build_contract,
        "--release",
    ];
    if is_workspace {
        params.push("--package");
        params.push(&host_crate.package);
    }
    let target_dir = target_dir.map(|dir| dir.to_string_lossy().into_owned());
    if let Some(target_dir) = &target_dir {
        params.push("--target-dir");
        params.push(target_dir);
    }
    cargo_checked(
        current_dir,
        "build",
        params,
        &[(ODRA_MODULE_ENV_KEY, odra_module)],
        capture_output,
    )
}

/// Build schema files.
///
/// The schema binary is built with the dev profile: its output does not depend on optimization,
/// and the host crate is rebuilt once per contract, so a release build (with LTO in a typical
/// contract project) would multiply that cost by the number of contracts.
///
/// Schemas are generated one at a time, unlike wasm files: a run costs about a second once the
/// crate is compiled, while a worker's target directory would hold a copy of the host
/// dependencies, close to 1 GB for a typical project.
pub fn cargo_generate_schema_files(current_dir: PathBuf, odra_module: &str, host_crate_name: &str) {
    let host_crate_name = host_crate_name.replace('-', "_");
    let gen_schema = format!("{host_crate_name}_build_schema");
    cargo_checked(
        &current_dir,
        "run",
        vec!["--bin", &gen_schema],
        &[(ODRA_MODULE_ENV_KEY, odra_module)],
        false,
    )
    .unwrap_or_else(|error| error.print_and_die());
}

/// Runs cargo test.
pub fn cargo_test_odra_vm<'a>(
    current_dir: PathBuf,
    filters: &'a TestFilters,
    mut args: Vec<&'a str>,
) {
    log::info("Running cargo test...");
    let mut tail_args = filters.as_args();
    tail_args.append(&mut args);
    cargo(current_dir, "test", tail_args);
}

/// Runs cargo test with backend features.
pub fn cargo_test_backend<'a>(
    project_root: PathBuf,
    backend_name: &str,
    filters: &'a TestFilters,
    mut args: Vec<&'a str>,
) {
    env::set_var(ODRA_BACKEND_ENV_KEY, backend_name);
    log::info("Running cargo test...");
    let mut tail_args = filters.as_args();
    tail_args.append(&mut args);
    cargo(project_root, "test", tail_args)
}

/// Runs cargo clean.
pub fn cargo_clean(current_dir: PathBuf) {
    log::info("Running cargo clean...");
    cargo(current_dir, "clean", vec![]);
}

/// Build wasm client.
pub fn cargo_build_wasm_client(current_dir: PathBuf, project_root: PathBuf) {
    log::info("Building WASM client...");
    let casper_contract_schemas_path = project_root
        .join("resources")
        .join("casper_contract_schemas");
    cargo(
        current_dir,
        "run",
        vec![
            "--bin",
            "codegen",
            "--features",
            "codegen",
            casper_contract_schemas_path.to_str().unwrap_or_default(),
            ".",
        ],
    );
}

/// Writes a content to a file at the given path.
pub fn write_to_file(path: PathBuf, content: &str) -> Result<(), Error> {
    let mut file = File::create(path)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}

/// Appends a content to a file at the given path.
pub fn append_file(path: PathBuf, content: &str) -> Result<(), Error> {
    let mut file = OpenOptions::new().append(true).open(path)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}

/// Replaces strings in a file.
pub fn replace_in_file(path: PathBuf, from: &str, to: &str) -> Result<(), Error> {
    let content = read_file_content(path.clone())?;
    let new_content = content.replace(from, to);
    write_to_file(path, new_content.as_str())?;
    Ok(())
}

/// Renames a file.
pub fn rename_file(path: PathBuf, new_name: &str) -> Result<(), Error> {
    let mut new_path = path.clone();
    new_path.pop();
    new_path.push(new_name);
    fs::rename(path, new_path)?;
    Ok(())
}

/// Loads a file to a string.
pub fn read_file_content(path: PathBuf) -> Result<String, Error> {
    fs::read_to_string(path).map_err(Error::IO)
}

// TODO: Is there a better way? A global static to hold that?
/// Extracts verbosity, by parsing bin arguments.
fn verbosity_arg<'a>() -> Option<&'a str> {
    let Cargo::Odra(args) = Cargo::parse();
    if args.verbose {
        Some("--verbose")
    } else if args.quiet {
        Some("--quiet")
    } else {
        None
    }
}
