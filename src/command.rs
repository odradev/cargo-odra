//! Module containing code that runs external commands.

use std::{
    env,
    fs::{self, File, OpenOptions},
    io::Write,
    path::PathBuf,
    process::{Command, ExitStatus},
};

use clap::Parser;
use Error::InvalidInternalCommand;

use crate::{
    actions::test::TestFilters,
    cli::Cargo,
    consts::{ODRA_BACKEND_ENV_KEY, ODRA_MODULE_ENV_KEY},
    errors::Error,
    log,
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

/// Quits if status of a command is not successful.
pub fn parse_command_result(status: ExitStatus, error: Error) {
    if !status.success() {
        error.print_and_die();
    }
}

/// Copies file
pub fn cp(source: PathBuf, target: PathBuf) {
    let status = Command::new("cp")
        .args([&source, &target])
        .status()
        .unwrap();

    parse_command_result(
        status,
        Error::CommandFailed(format!(
            "Couldn't copy {} to {}",
            source.display(),
            target.display()
        )),
    );
}

/// Remove a directory.
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

/// Runs wasm-strip and wasm-opt on a given contract's wasm file.
pub fn process_wasm(contract_name: &str, project_root: PathBuf) {
    let mut cmd = Command::new("wasm-opt");
    cmd.current_dir(project_root.clone());

    cmd.arg("--signext-lowering");

    if needs_bulk_memory_flags() {
        cmd.arg("--enable-bulk-memory")
            .arg("--llvm-memory-copy-fill-lowering");
    }

    let command = cmd
        .arg(paths::wasm_path_in_wasm_dir(contract_name, &project_root))
        .arg("-o")
        .arg(paths::wasm_path_in_wasm_dir(contract_name, &project_root))
        .status();

    if command.is_err() || !command.unwrap().success() {
        Error::WasmoptDidNotFinish.print_and_die();
    }

    let command = Command::new("wasm-strip")
        .current_dir(project_root.clone())
        .arg(paths::wasm_path_in_wasm_dir(contract_name, &project_root))
        .status();

    if command.is_err() || !command.unwrap().success() {
        Error::WasmstripDidNotFinish.print_and_die();
    }
}

/// Runs cargo with given args.
fn cargo(current_dir: PathBuf, command: &str, tail_args: Vec<&str>) {
    let mut args = vec![command];

    if let Some(verbosity) = verbosity_arg() {
        args.push(verbosity);
    }

    for arg in tail_args {
        args.push(arg);
    }

    let command = Command::new("cargo")
        .current_dir(current_dir)
        .args(args.as_slice())
        .status()
        .unwrap();

    parse_command_result(
        command,
        Error::CommandFailed(format!("Couldn't run cargo with args {args:?}")),
    );
}

/// Build wasm files.
pub fn cargo_build_wasm_files(
    current_dir: PathBuf,
    contract_name: &str,
    build_bin: &str,
    is_workspace: bool,
    crate_name: String,
) {
    env::set_var(ODRA_MODULE_ENV_KEY, contract_name);
    let mut params = vec![
        "--target",
        "wasm32-unknown-unknown",
        "--bin",
        build_bin,
        "--release",
    ];
    if is_workspace {
        params.push("--package");
        params.push(crate_name.as_str());
    }
    cargo(current_dir, "build", params);
}

/// Build schema files.
pub fn cargo_generate_schema_files(current_dir: PathBuf, contract_name: &str, module_name: &str) {
    let module_name = module_name.replace('-', "_");
    env::set_var(ODRA_MODULE_ENV_KEY, contract_name);
    let gen_schema = format!("{module_name}_build_schema");
    cargo(current_dir, "run", vec!["--bin", &gen_schema, "--release"]);
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
