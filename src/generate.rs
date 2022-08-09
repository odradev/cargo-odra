use crate::odra_toml::OdraConf;
use crate::GenerateCommand;
use convert_case::{Case, Casing};
use prettycli::info;
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;

pub struct Generate {
    generate: GenerateCommand,
}

impl Generate {
    pub fn new(generate: GenerateCommand) -> Generate {
        Generate { generate }
    }

    pub(crate) fn generate_contract(&self) {
        info(&format!("Contract: {}", self.generate.contract_name));
        let contract_body = attohttpc::get(
            "https://raw.githubusercontent.com/odradev/odra-template/master/src/flipper.rs",
        )
        .send()
        .unwrap()
        .text()
        .unwrap();
        let contract_body = contract_body.replace(
            "Flipper",
            self.generate
                .contract_name
                .to_case(Case::UpperCamel)
                .as_str(),
        );
        let contract_body = contract_body.replace(
            "flipper",
            self.generate.contract_name.to_case(Case::Lower).as_str(),
        );
        fs::write(
            format!("src/{}.rs", &self.generate.contract_name),
            contract_body,
        )
        .unwrap();
        let mut lib_rs = OpenOptions::new()
            .write(true)
            .append(true)
            .open("src/lib.rs")
            .unwrap();
        let mod_line = format!("pub mod {};", &self.generate.contract_name);
        let use_line = format!(
            "pub use {}::{{{}, {}}};",
            &self.generate.contract_name,
            &self.generate.contract_name.to_case(Case::UpperCamel),
            self.generate.contract_name.to_case(Case::UpperCamel) + "Ref"
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
            OdraConf::load().name,
            self.generate.contract_name.to_case(Case::UpperCamel)
        );
        writeln!(odra_toml).unwrap();
        writeln!(
            odra_toml,
            "{} = {{ path = \"src/{}.rs\", name = \"{}\", fqn = \"{}\"}}",
            self.generate.contract_name,
            self.generate.contract_name,
            self.generate.contract_name,
            fqn
        )
        .unwrap();
        lib_rs.flush().unwrap();
    }
}
