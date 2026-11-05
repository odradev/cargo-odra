use std::{fs, path::Path, process::Command};

use crate::{
    command,
    consts::{ODRA_GITHUB_API_DATA, ODRA_LATEST_RELEASE_URL},
    errors::Error,
    odra_toml::{Contract, ContractSource},
    paths::to_snake_case,
    project::Project,
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
            .filter(|c| names.contains(&to_snake_case(c.struct_name())))
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
            .any(|c| to_snake_case(c.struct_name()) == *contract_name)
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

/// Checks that every contract coming from a dependency can end up in the built wasm.
///
/// Rust links a dependency's object files only when something in the crate being built
/// references them, so a host crate that depends on, say, `odra-modules` but never mentions it
/// produces a wasm without a single entry point - and a linker error for the schema binary.
/// The reference is looked for in the host crate's own Rust sources.
pub fn validate_external_contracts(project: &Project, contracts: &[Contract]) {
    for contract in contracts {
        let ContractSource::Dependency(dependency) = contract.source(project) else {
            continue;
        };
        let host = contract.host_crate(project);
        if !crate_is_referenced(&host.root, &dependency) {
            Error::HostCrateDoesNotUseDependency(contract.fqn.clone(), dependency, host.name)
                .print_and_die();
        }
    }
}

/// Looks for `crate_name` as a whole word in the Rust sources of a crate.
fn crate_is_referenced(crate_root: &Path, crate_name: &str) -> bool {
    let pattern = regex::Regex::new(&format!(r"\b{}\b", regex::escape(crate_name)))
        .expect("a crate name is a valid regex literal");
    ["*.rs", "src/**/*.rs", "bin/**/*.rs"]
        .iter()
        .flat_map(|glob_pattern| {
            let glob_pattern = crate_root.join(glob_pattern).to_string_lossy().into_owned();
            glob::glob(&glob_pattern)
                .map(|paths| paths.filter_map(Result::ok).collect::<Vec<_>>())
                .unwrap_or_default()
        })
        .any(|path| {
            fs::read_to_string(path)
                .map(|source| pattern.is_match(&source))
                .unwrap_or(false)
        })
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
        false => remove_extra_spaces(&names_string)
            .map(|string| string.split(' ').map(to_snake_case).collect::<Vec<_>>()),
    }
}

/// Returns the tag of the latest Odra release.
///
/// Resolved from the `releases/latest` redirect on github.com, which is not rate limited.
/// The REST API is only a fallback: unauthenticated calls share 60 requests per hour per
/// IP, which shared CI runners exhaust routinely. A `GITHUB_TOKEN` or `GH_TOKEN` in the
/// environment is used for the fallback when present.
pub fn odra_latest_version() -> String {
    let redirect_error = match latest_version_from_redirect() {
        Ok(version) => return version,
        Err(error) => error,
    };
    match latest_version_from_api() {
        Ok(version) => version,
        Err(api_error) => Error::FailedToFetchLatestVersion(format!(
            "{ODRA_LATEST_RELEASE_URL}: {redirect_error}; {ODRA_GITHUB_API_DATA}: {api_error}"
        ))
        .print_and_die(),
    }
}

fn latest_version_from_redirect() -> Result<String, String> {
    let response = ureq::get(ODRA_LATEST_RELEASE_URL)
        .config()
        .max_redirects(0)
        .max_redirects_will_error(false)
        .http_status_as_error(false)
        .build()
        .call()
        .map_err(|error| error.to_string())?;
    let location = response
        .headers()
        .get("location")
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| format!("expected a redirect, got HTTP {}", response.status()))?;
    tag_from_release_location(location)
        .ok_or_else(|| format!("unexpected redirect target `{location}`"))
}

/// Extracts the tag from a GitHub release page URL such as
/// `https://github.com/odradev/odra/releases/tag/2.9.1`.
pub fn tag_from_release_location(location: &str) -> Option<String> {
    let (_, tag) = location.rsplit_once("/releases/tag/")?;
    let tag = tag.trim_end_matches('/');
    (!tag.is_empty()).then(|| tag.to_string())
}

fn latest_version_from_api() -> Result<String, String> {
    let mut request = ureq::get(ODRA_GITHUB_API_DATA);
    let token = ["GITHUB_TOKEN", "GH_TOKEN"]
        .iter()
        .find_map(|name| std::env::var(name).ok())
        .filter(|token| !token.is_empty());
    if let Some(token) = token {
        request = request.header("Authorization", format!("Bearer {token}"));
    }
    let response: serde_json::Value = request
        .call()
        .map_err(|error| error.to_string())?
        .body_mut()
        .read_json()
        .map_err(|error| error.to_string())?;
    response["tag_name"]
        .as_str()
        .map(str::to_string)
        .ok_or_else(|| "response has no `tag_name`".to_string())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::{crate_is_referenced, tag_from_release_location};

    #[test]
    fn finds_a_crate_referenced_in_the_sources_of_the_host_crate() {
        let root = std::env::temp_dir().join(format!("cargo-odra-refs-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("src")).unwrap();
        fs::create_dir_all(root.join("bin")).unwrap();
        fs::write(root.join("src/lib.rs"), "pub mod flipper;\n").unwrap();
        fs::write(root.join("bin/build_contract.rs"), "use flipper;\n").unwrap();

        assert!(!crate_is_referenced(&root, "odra_modules"));
        // A crate whose name is only a prefix of what is in the sources does not count.
        assert!(!crate_is_referenced(&root, "flip"));
        assert!(crate_is_referenced(&root, "flipper"));

        fs::write(
            root.join("bin/build_contract.rs"),
            "use flipper;\nuse odra_modules;\n",
        )
        .unwrap();
        assert!(crate_is_referenced(&root, "odra_modules"));

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn extracts_the_tag_from_a_release_page_url() {
        assert_eq!(
            tag_from_release_location("https://github.com/odradev/odra/releases/tag/2.9.1"),
            Some("2.9.1".to_string())
        );
        assert_eq!(
            tag_from_release_location("https://github.com/odradev/odra/releases/tag/v3.0.0-rc.1/"),
            Some("v3.0.0-rc.1".to_string())
        );
    }

    #[test]
    fn rejects_urls_that_are_not_release_pages() {
        assert_eq!(
            tag_from_release_location("https://github.com/odradev/odra/releases"),
            None
        );
        assert_eq!(
            tag_from_release_location("https://github.com/odradev/odra/releases/tag/"),
            None
        );
    }
}
