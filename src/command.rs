use std::fs;
use std::process::{Command, ExitStatus};

pub fn parse_command_result(status: ExitStatus, msg: &str) {
    if !status.success() {
        panic!("{}", msg);
    }
}

pub fn cp(source: &str, target: &str) {
    let status = Command::new("cp").args([source, target]).status().unwrap();

    parse_command_result(status, &format!("Couldn't copy {} to {}", source, target));
}

pub fn fmt(folder: &str) {
    let status = Command::new("cargo")
        .args(["fmt"])
        .current_dir(folder)
        .status()
        .unwrap();

    parse_command_result(status, &format!("Couldn't run cargo fmt in {}", folder));
}

pub fn mkdir(path: &str) {
    fs::create_dir_all(path).unwrap();
}

pub fn wasm_strip(contract_name: &str) {
    let command = Command::new("wasm-strip")
        .current_dir("wasm")
        .arg(format!("{}.wasm", contract_name))
        .status()
        .expect("Couldn't run wasmstrip");

    match command.success() {
        true => {}
        false => {
            println!("There was an error while running wasmstrip - Continuing anyway...");
        }
    }
}
