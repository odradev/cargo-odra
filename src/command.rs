//! File containing code that runs external commands
use prettycli::{critical, error, warn};
use std::fs;
use std::process::{exit, Command, ExitStatus};

/// Returns output of a command as a String
pub fn command_output(command: &str) -> String {
    let mut split_command: Vec<&str> = command.split(' ').collect();
    let program = *split_command.first().unwrap_or_else(|| {
        error(&format!("Invalid command {}", command));
        exit(1);
    });

    let args: Vec<&str> = split_command.drain(1..).collect();

    let output = Command::new(program).args(args).output().unwrap();

    std::str::from_utf8(output.stdout.as_slice())
        .unwrap()
        .to_string()
}

/// Quits if status of a command is not succesfull
pub fn parse_command_result(status: ExitStatus, msg: &str) {
    if !status.success() {
        critical(msg);
        exit(1);
    }
}

/// Copies file
pub fn cp(source: &str, target: &str) {
    let status = Command::new("cp").args([source, target]).status().unwrap();

    parse_command_result(status, &format!("Couldn't copy {} to {}", source, target));
}

/// Runs cargo fmt
pub fn fmt(folder: &str) {
    let status = Command::new("cargo")
        .args(["fmt"])
        .current_dir(folder)
        .status()
        .unwrap();

    parse_command_result(status, &format!("Couldn't run cargo fmt in {}", folder));
}

/// Creates a directory
pub fn mkdir(path: &str) {
    fs::create_dir_all(path).unwrap();
}

/// Runs wasm-strip
pub fn wasm_strip(contract_name: &str) {
    let command = Command::new("wasm-strip")
        .current_dir("wasm")
        .arg(format!("{}.wasm", contract_name))
        .status();

    if command.is_ok() && command.unwrap().success() {
        return;
    }

    warn("There was an error while running wasmstrip - is it installed? Continuing anyway...");
}

/// Runs cargo with given args
pub fn cargo(current_dir: String, args: Vec<&str>) {
    let command = Command::new("cargo")
        .current_dir(current_dir)
        .args(args.as_slice())
        .status()
        .unwrap();

    parse_command_result(command, &format!("Couldn't run cargo with args {:?}", args));
}
