//! Module containing code that runs external commands
use crate::errors::Error;
use crate::{cli::Cargo, log};
use clap::Parser;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, ExitStatus};
use Error::InvalidInternalCommand;

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

/// Quits if status of a command is not successful
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

/// Creates a directory.
pub fn mkdir(path: PathBuf) {
    fs::create_dir_all(path).unwrap();
}

/// Runs wasm-strip.
pub fn wasm_strip(contract_name: &str) {
    let command = Command::new("wasm-strip")
        .current_dir("wasm")
        .arg(format!("{}.wasm", contract_name))
        .status();

    if command.is_ok() && command.unwrap().success() {
        return;
    }

    Error::WasmstripNotInstalled.print_and_die();
}

/// Runs cargo with given args
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
        Error::CommandFailed(format!("Couldn't run cargo with args {:?}", args)),
    );
}

/// Build wasm files.
pub fn cargo_build_wasm_files(current_dir: PathBuf, contract_name: &str) {
    cargo(
        current_dir,
        "build",
        vec![
            "--target",
            "wasm32-unknown-unknown",
            "--bin",
            contract_name,
            "--release",
            "--no-default-features",
            "--target-dir",
            "../target",
        ],
    );
}

/// Build wasm sources.
pub fn cargo_build_wasm_sources(current_dir: PathBuf, contract_name: &str) {
    cargo(
        current_dir,
        "run",
        vec![
            "--bin",
            format!("{}_build", contract_name).as_str(),
            "--release",
            "--no-default-features",
            "--target-dir",
            "../target",
        ],
    );
}

/// Update a cargo module.
pub fn cargo_update(current_dir: PathBuf) {
    cargo(current_dir, "update", vec![]);
}

/// Runs cargo fmt.
pub fn cargo_fmt(current_dir: PathBuf) {
    cargo(current_dir, "fmt", vec![]);
}

pub fn cargo_test_mock_vm(current_dir: PathBuf, args: Vec<&str>) {
    log::info("Running cargo test...");
    cargo(current_dir, "test", args);
}

pub fn cargo_test_backend(current_dir: PathBuf, backend_name: &str, tail_args: Vec<&str>) {
    log::info("Running cargo test...");
    let mut args = vec!["--no-default-features", "--features", backend_name];
    for arg in tail_args {
        args.push(arg);
    }

    cargo(current_dir, "test", args)
}

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
