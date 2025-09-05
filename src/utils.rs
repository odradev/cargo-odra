use std::process::Command;

use ureq::serde_json;

use crate::{
    command, consts::ODRA_GITHUB_API_DATA, errors::Error, odra_toml::Contract,
    paths::to_camel_case, project::Project,
};

/// Check if wasm32-unknown-unknown target is installed.
pub fn check_target_requirements() {
    if !command::command_output("rustup target list --installed").contains("wasm32-unknown-unknown")
    {
        Error::WasmTargetNotInstalled.print_and_die();
    }
}

/// Check if wasm-pack is installed.
pub fn check_wasm_pack() {
    let result = Command::new("wasm-pack")
        .arg("--version")
        .status()
        .unwrap_or_else(|_| {
            Error::WasmPackNotInstalled.print_and_die();
        });

    if !result.success() {
        Error::WasmPackNotInstalled.print_and_die()
    }
}

/// Returns list of contract to process.
pub fn contracts(project: &Project, names_string: String) -> Result<Vec<Contract>, &'static str> {
    let names = parse_contracts_names(names_string)?;
    let odra_toml = project.odra_toml();
    Ok(match names.is_empty() {
        true => odra_toml.contracts,
        false => odra_toml
            .contracts
            .into_iter()
            .filter(|c| names.contains(&c.struct_name()))
            .collect(),
    })
}

/// Check if contract name argument is valid if set.
pub fn validate_contract_name_argument(project: &Project, names_string: String) {
    let names = parse_contracts_names(names_string).unwrap_or_default();
    names.iter().for_each(|contract_name| {
        if !project
            .odra_toml()
            .contracts
            .iter()
            .any(|c| c.struct_name() == *contract_name)
        {
            Error::ContractNotFound(contract_name.clone()).print_and_die();
        }
    });
}

/// Validate if contract names are unique.
pub fn validate_contract_names(project: &Project) {
    project.odra_toml().contracts.iter().for_each(|contract| {
        if project
            .odra_toml()
            .contracts
            .iter()
            .filter(|c| c.struct_name() == contract.struct_name())
            .count()
            > 1
        {
            Error::ContractDuplicate(contract.struct_name()).print_and_die();
        }
    });
}

fn remove_extra_spaces(input: &str) -> Result<String, &'static str> {
    // Ensure there are no other separators
    if input.chars().any(|c| c.is_whitespace() && c != ' ') {
        return Err("Input contains non-space whitespace characters");
    }

    let trimmed = input.split_whitespace().collect::<Vec<&str>>().join(" ");
    Ok(trimmed)
}

fn parse_contracts_names(names_string: String) -> Result<Vec<String>, &'static str> {
    match names_string.is_empty() {
        true => Ok(vec![]),
        false => remove_extra_spaces(&names_string).map(|string| {
            string
                .split(' ')
                .map(ToString::to_string)
                .map(to_camel_case)
                .collect::<Vec<_>>()
        }),
    }
}

pub fn odra_latest_version() -> String {
    let response: serde_json::Value = ureq::get(ODRA_GITHUB_API_DATA)
        .call()
        .unwrap_or_else(|_| {
            Error::FailedToFetchTemplate(ODRA_GITHUB_API_DATA.to_string()).print_and_die()
        })
        .into_json()
        .unwrap_or_else(|_| {
            Error::FailedToParseTemplate(ODRA_GITHUB_API_DATA.to_string()).print_and_die()
        });
    response["tag_name"].as_str().unwrap().to_string()
}
