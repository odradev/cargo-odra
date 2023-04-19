//! Constants used by cargo odra

/// Casper backend name.
pub const ODRA_CASPER_BACKEND: &str = "casper";

/// Template repository path.
pub const ODRA_TEMPLATE_GH_REPO: &str = "https://github.com/odradev/odra.git";

/// Template raw repository path.
pub const ODRA_TEMPLATE_GH_RAW_REPO: &str = "https://raw.githubusercontent.com/odradev/odra";

/// Default odra-template git branch.
pub const ODRA_TEMPLATE_GH_BRANCH: &str = "feature/cargo-odra-templates";

/// Default template name.
pub const ODRA_TEMPLATE_DEFAULT_TEMPLATE: &str = "full";

/// WASM Path
pub const ODRA_WASM_PATH_ENV_KEY: &str = "ODRA_WASM_PATH";

/// WASM Source builder template.
pub const WASM_SOURCE_BUILDER: &str = "wasm_source_builder.rs";

/// Module template.
pub const MODULE_TEMPLATE: &str = "module.rs";

/// Module register snippet.
pub const MODULE_REGISTER: &str = "module_register.rs";
