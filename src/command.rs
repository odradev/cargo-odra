//! Module containing code that runs external commands
use crate::errors::Error;
use crate::Cargo;
use clap::Parser;
use std::fs;
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
pub fn cp(source: &str, target: &str) {
    let status = Command::new("cp").args([source, target]).status().unwrap();

    parse_command_result(
        status,
        Error::CommandFailed(format!("Couldn't copy {} to {}", source, target)),
    );
}

/// Runs cargo fmt.
pub fn fmt(folder: &str) {
    let status = Command::new("cargo")
        .args(["fmt"])
        .current_dir(folder)
        .status()
        .unwrap();

    parse_command_result(
        status,
        Error::CommandFailed(format!("Couldn't run cargo fmt in {}", folder)),
    );
}

/// Creates a directory.
pub fn mkdir(path: &str) {
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

// @TODO: Use PathBuf
/// Runs cargo with given args
pub fn cargo(current_dir: String, args: Vec<&str>) {
    let args = add_verbosity(args);
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

fn add_verbosity(mut command_args: Vec<&str>) -> Vec<&str> {
    let Cargo::Odra(args) = Cargo::parse();
    if args.verbose {
        let mut result = vec!["--verbose"];
        result.append(&mut command_args);
        return result;
    } else if args.quiet {
        let mut result = vec!["--quiet"];
        result.append(&mut command_args);
        return result;
    }

    command_args
}

// fn add_target_dir(mut command_args: Vec<&str>) -> Vec<&str> {
//     command_args.push("--ta")
// }
