use crate::Generate;
use cargo_generate::{GenerateArgs, TemplatePath, Vcs};
use convert_case::{Case, Casing};
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;

pub(crate) fn new_project(name: Option<String>) {
    cargo_generate::generate(GenerateArgs {
        template_path: TemplatePath {
            auto_path: Some("odradev/odra-template".to_string()),
            subfolder: None,
            git: None,
            branch: None,
            path: None,
            favorite: None,
        },
        list_favorites: false,
        name,
        force: false,
        verbose: false,
        template_values_file: None,
        silent: false,
        config: None,
        vcs: Vcs::None,
        lib: false,
        bin: false,
        ssh_identity: None,
        define: vec![],
        init: false,
        destination: None,
        force_git_init: false,
        allow_commands: false,
    })
    .unwrap();
}

pub(crate) fn init_project() {
    cargo_generate::generate(GenerateArgs {
        template_path: TemplatePath {
            auto_path: Some("odradev/odra-template".to_string()),
            subfolder: None,
            git: None,
            branch: None,
            path: None,
            favorite: None,
        },
        list_favorites: false,
        name: None,
        force: false,
        verbose: false,
        template_values_file: None,
        silent: false,
        config: None,
        vcs: Vcs::None,
        lib: false,
        bin: false,
        ssh_identity: None,
        define: vec![],
        init: true,
        destination: None,
        force_git_init: false,
        allow_commands: false,
    })
    .unwrap();
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
    fs::write(format!("src/{}.rs", &generate.contract_name), contract_body).unwrap();
    let mut lib_rs = OpenOptions::new()
        .write(true)
        .append(true)
        .open("src/lib.rs")
        .unwrap();
    let mod_line = format!("pub mod {};", &generate.contract_name);
    let use_line = format!(
        "pub use {}::{};",
        &generate.contract_name,
        &generate.contract_name.to_case(Case::UpperCamel)
    );
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
