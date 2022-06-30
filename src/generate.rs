use crate::Generate;
use convert_case::{Case, Casing};
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::os::unix::process::CommandExt;
use std::process::Command;

pub(crate) fn new_project() {
    Command::new("cargo")
        .args(["generate", "odradev/odra-template"])
        .exec();
}

pub(crate) fn init_project() {
    Command::new("cargo")
        .args(["generate", "odradev/odra-template", "--init"])
        .exec();
}

pub(crate) fn generate_contract(generate: &Generate) {
    println!("Contract: {}", generate.contract_name);
    let contract_body = attohttpc::get(
        "https://raw.githubusercontent.com/odradev/odra-template/master/src/flipper.rs",
    )
    .send()
    .unwrap()
    .text()
    .unwrap();
    let contract_body = contract_body.replace(
        "Flipper",
        generate.contract_name.to_case(Case::UpperCamel).as_str(),
    );
    let contract_body = contract_body.replace(
        "flipper",
        generate.contract_name.to_case(Case::Lower).as_str(),
    );
    fs::write(
        ("src/".to_string() + &generate.contract_name + ".rs").as_str(),
        contract_body,
    )
    .unwrap();
    let mut lib_rs = OpenOptions::new()
        .write(true)
        .append(true)
        .open("src/lib.rs")
        .unwrap();
    let mod_line = "pub mod ".to_string() + &generate.contract_name + ";";
    let use_line = "pub use ".to_string()
        + &generate.contract_name
        + "::"
        + &generate.contract_name.to_case(Case::UpperCamel)
        + ";";
    writeln!(lib_rs).unwrap();
    writeln!(lib_rs, "{}", mod_line).unwrap();
    writeln!(lib_rs, "{}", use_line).unwrap();
    writeln!(lib_rs).unwrap();
    lib_rs.flush().unwrap();

    let mut odra_toml = OpenOptions::new()
        .write(true)
        .append(true)
        .open("Odra.toml")
        .unwrap();
    let fqn = format!(
        "{}::{}",
        crate::odra_toml::load_odra_conf().name,
        generate.contract_name.to_case(Case::UpperCamel)
    );
    writeln!(odra_toml).unwrap();
    writeln!(
        odra_toml,
        "{} = {{ path = \"src/{}.rs\", name = \"{}\", fqn = \"{}\"}}",
        generate.contract_name, generate.contract_name, generate.contract_name, fqn
    )
    .unwrap();
    lib_rs.flush().unwrap();
}
