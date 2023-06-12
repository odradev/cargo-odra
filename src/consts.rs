//! Constants used by cargo odra

/// Casper backend name.
pub const ODRA_CASPER_BACKEND: &str = "casper";

/// Template repository path.
pub const ODRA_TEMPLATE_GH_REPO: &str = "https://github.com/odradev/odra.git";

/// Template raw repository path.
pub const ODRA_TEMPLATE_GH_RAW_REPO: &str = "https://raw.githubusercontent.com/odradev/odra";

pub const ODRA_GITHUB_API_DATA: &str = "https://api.github.com/repos/odradev/odra/releases/latest";

/// Default template name.
pub const ODRA_TEMPLATE_DEFAULT_TEMPLATE: &str = "full";

/// WASM Path
pub const ODRA_WASM_PATH_ENV_KEY: &str = "ODRA_WASM_PATH";

/// WASM Source builder template.
pub const WASM_SINGLE_SOURCE_BUILDER: &str = "contracts_builder/wasm_source_builder";

/// WASM Source builder helper template.
pub const MATCH_CONTRACT_NAME: &str = "contracts_builder/match_contract_name";

/// WASM Source builder helper template.
pub const GEN_CONTRACT_MOD: &str = "contracts_builder/gen_contract_mod";

/// Module template.
pub const MODULE_TEMPLATE: &str = "module";

/// Module register snippet.
pub const MODULE_REGISTER: &str = "module_register";
