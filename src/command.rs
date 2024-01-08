//! Module containing code that runs external commands.

use std::{
    env,
    fs::{self, File, OpenOptions},
    io::{self, Write},
    path::PathBuf,
    process::{Command, ExitStatus},
};

use clap::Parser;
use Error::InvalidInternalCommand;

use crate::{
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
pub fn mkdir(path: PathBuf) {
    fs::create_dir_all(path).unwrap();
}

/// Runs wasm-strip.
pub fn wasm_strip(contract_name: &str, project_root: PathBuf) {
    let command = Command::new("wasm-strip")
        .current_dir(project_root.clone())
        .arg(paths::wasm_path_in_wasm_dir(contract_name, project_root))
        .status();

    if command.is_ok() && command.unwrap().success() {
        return;
    }

    Error::WasmstripNotInstalled.print_and_die();
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
pub fn cargo_build_wasm_files(current_dir: PathBuf, contract_name: &str, module_name: &str) {
    env::set_var(ODRA_MODULE_ENV_KEY, contract_name);
    let build_contract = format!("{}_build_contract", module_name);
    cargo(
        current_dir,
        "build",
        vec![
            "--target",
            "wasm32-unknown-unknown",
            "--bin",
            &build_contract,
            "--release",
        ],
    );
}

/// Update a cargo module.
pub fn cargo_update(current_dir: PathBuf) {
    cargo(current_dir, "update", vec![]);
}

/// Runs cargo test.
pub fn cargo_test_mock_vm(current_dir: PathBuf, mut args: Vec<&str>) {
    log::info("Running cargo test...");
    let mut tail_args = vec!["--lib"];
    tail_args.append(&mut args);
    cargo(current_dir, "test", tail_args);
}

/// Runs cargo test with backend features.
pub fn cargo_test_backend(project_root: PathBuf, backend_name: &str, mut args: Vec<&str>) {
    env::set_var(ODRA_BACKEND_ENV_KEY, backend_name);
    log::info("Running cargo test...");
    let mut tail_args = vec!["--lib"];
    tail_args.append(&mut args);
    cargo(project_root, "test", tail_args)
}

/// Runs cargo clean.
pub fn cargo_clean(current_dir: PathBuf) {
    log::info("Running cargo clean...");
    cargo(current_dir, "clean", vec![]);
}

/// Writes a content to a file at the given path.
pub fn write_to_file(path: PathBuf, content: &str) {
    let mut file = File::create(path).unwrap();
    file.write_all(content.as_bytes()).unwrap();
}

/// Appends a content to a file at the given path.
pub fn append_file(path: PathBuf, content: &str) {
    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .open(path)
        .unwrap();

    file.write_all(content.as_bytes()).unwrap();
}

/// Replaces strings in a file.
pub fn replace_in_file(path: PathBuf, from: &str, to: &str) {
    let content = read_file_content(path.clone()).unwrap();
    let new_content = content.replace(from, to);
    write_to_file(path, new_content.as_str());
}

/// Loads a file to a string.
pub fn read_file_content(path: PathBuf) -> io::Result<String> {
    fs::read_to_string(path)
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
