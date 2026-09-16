//! Constants used by cargo odra

/// Casper backend name.
pub const ODRA_CASPER_BACKEND: &str = "casper";

/// Odra backend env key
pub const ODRA_BACKEND_ENV_KEY: &str = "ODRA_BACKEND";

/// Odra module env key
pub const ODRA_MODULE_ENV_KEY: &str = "ODRA_MODULE";

/// Template repository path.
pub const ODRA_TEMPLATE_GH_REPO: &str = "https://github.com/odradev/odra.git";

/// Template raw repository path.
pub const ODRA_TEMPLATE_GH_RAW_REPO: &str = "https://raw.githubusercontent.com/odradev/odra";

/// Redirects to the tag page of the latest Odra release. Unlike the REST API below it is
/// not rate limited, so it is the primary way to learn the latest version.
pub const ODRA_LATEST_RELEASE_URL: &str = "https://github.com/odradev/odra/releases/latest";

/// REST API endpoint for the latest Odra release. Unauthenticated calls share a limit of
/// 60 requests per hour per IP, which CI runners exhaust routinely; only a fallback.
pub const ODRA_GITHUB_API_DATA: &str = "https://api.github.com/repos/odradev/odra/releases/latest";

/// Default template name.
pub const ODRA_TEMPLATE_DEFAULT_TEMPLATE: &str = "full";

/// Module template.
pub const MODULE_TEMPLATE: &str = "flipper";

/// Module register snippet.
pub const MODULE_REGISTER: &str = "module_register";

/// Path of templates.json file in Odra repository.
pub const TEMPLATES_JSON_PATH: &str = "templates/templates.json";
