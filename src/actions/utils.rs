use crate::{command, errors::Error, odra_toml::Contract, project::Project};

/// Check if wasm32-unknown-unknown target is installed.
pub fn check_target_requirements() {
    if !command::command_output("rustup target list --installed").contains("wasm32-unknown-unknown")
    {
        Error::WasmTargetNotInstalled.print_and_die();
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
                .collect::<Vec<_>>()
        }),
    }
}
