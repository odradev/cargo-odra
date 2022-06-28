mod builder;
mod backend;
mod odra_toml;
mod cargo_toml;

use std::ffi::OsString;
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::Command;
use clap::{Parser, Subcommand};
use convert_case::{Case, Casing};

#[derive(Parser)]
#[clap(name = "cargo")]
#[clap(bin_name = "cargo")]
enum Cargo {
    Odra(Odra),
}

#[derive(clap::Args)]
#[clap(author, version, about, long_about = None)]
struct Odra {
    #[clap(subcommand)]
    subcommand: OdraSubcommand,
}

#[derive(Subcommand)]
enum OdraSubcommand {
    New(New),
    Build(Build),
    Test(Test),
    Generate(Generate),
}

#[derive(clap::Args)]
struct New {

}

#[derive(clap::Args)]
struct Init {

}

#[derive(clap::Args)]
struct Build {

}

#[derive(clap::Args, Debug)]
struct Test {
    #[clap(value_parser, long, short)]
    backend: Option<String>,
    #[clap(value_parser, long, short)]
    passthrough: Option<Vec<OsString>>
}

#[derive(clap::Args, Debug)]
struct Generate {
    #[clap(value_parser, long, short)]
    contract_name: String,
}

fn main() {
    let Cargo::Odra(args) = Cargo::parse();
    match args.subcommand {
        OdraSubcommand::Build(_) => {
            println!("Build!");
        }
        OdraSubcommand::Test(test) => {
            match test.backend {
                None => {test_mock_vm(&test);}
                Some(_) => {test_backend(&test);}
            }
        }
        OdraSubcommand::Generate(generate) => {
            generate_contract(&generate);
        }
        OdraSubcommand::New(_) => {
            new_project();
        }
    }
}

fn test_backend(test: &Test) {
    let backend = test.backend.clone().unwrap();

    if !Path::new(".backend").is_dir() {
        let repo_uri = format!("git@github.com:odradev/odra-{}.git", backend);
        backend::pull_backend(&repo_uri);
    }

    if !Path::new("target/debug/libodra_test_env.so").exists() {
        backend::build_backend(&backend);
    }

    let odra_conf = odra_toml::load_odra_conf();

    builder::prepare_builder(&backend, &odra_conf);

    cargo_toml::build_cargo_toml(&backend, &odra_conf);

    builder::build_wasm(&odra_conf);

    builder::copy_wasm_files(&odra_conf);

    let mut test_args = get_test_args(test);
    test_args.append(&mut vec!["--no-default-features", "--features=wasm-test"]);

    Command::new("cargo")
        .args(test_args)
        .exec();
}


fn test_mock_vm(test: &Test) {
    let test_args = get_test_args(test);

    Command::new("cargo")
    .args(test_args)
    .exec();
}

fn get_test_args(test: &Test) -> Vec<&str> {
    let mut test_args = vec!["test"];
    match &test.passthrough {
        None => {}
        Some(passthrough) => {
            let passthrough = passthrough.first().unwrap().as_os_str().to_str().unwrap();
            let mut vec: Vec<&str> = passthrough.split(' ').collect();
            test_args.append(&mut vec);
        }
    }
    test_args
}

fn new_project() {
    Command::new("cargo")
        .args(["generate", "odradev/odra-template"])
        .exec();
}

fn generate_contract(generate: &Generate) {
    println!("Contract: {}", generate.contract_name);
    let contract_body = attohttpc::get("https://raw.githubusercontent.com/odradev/odra-template/master/src/flipper.rs").send().unwrap().text().unwrap();
    let contract_body = contract_body.replace("Flipper", generate.contract_name.to_case(Case::UpperCamel).as_str());
    let contract_body = contract_body.replace("flipper", generate.contract_name.to_case(Case::Lower).as_str());
    fs::write(("src/".to_string() + &generate.contract_name + ".rs").as_str(), contract_body).unwrap();
    let mut lib_rs = OpenOptions::new()
        .write(true)
        .append(true)
        .open("src/lib.rs")
        .unwrap();
    let mod_line = "pub mod ".to_string() + &generate.contract_name + ";";
    let use_line = "pub use ".to_string() + &generate.contract_name + "::" + &generate.contract_name.to_case(Case::UpperCamel) + ";";
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
    let fqn = format!("{}::{}", odra_toml::load_odra_conf().name, generate.contract_name.to_case(Case::UpperCamel));
    writeln!(odra_toml).unwrap();
    writeln!(odra_toml, "{} = {{ path = \"src/{}.rs\", name = \"{}\", fqn = \"{}\"}}", generate.contract_name, generate.contract_name, generate.contract_name, fqn).unwrap();
    lib_rs.flush().unwrap();
}

#[cfg(test)]
mod test {
    #[test]
    fn test_smth() {
        assert!(true);
    }
}