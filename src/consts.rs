//! Constants used by cargo odra

/// Name of the Odra framework crate
pub const ODRA_CRATE_NAME: &str = "odra";
/// Name of the file which holds Odra configuration
pub const ODRA_TOML_FILENAME: &str = "Odra.toml";

///This is a platform-specific file prefix.
///
/// In Unix-based systems the convention is to start the file name with "lib".
/// Windows does not have such a convention.
#[cfg(unix)]
pub const PLATFORM_FILE_PREFIX: &str = "lib";
///This is a platform-specific file prefix.
///
/// In Unix-based systems the convention is to start the file name with "lib".
/// Windows does not have such a convention.
#[cfg(windows)]
pub const PLATFORM_FILE_PREFIX: &str = "";

///Dynamic link library file extension specific to the platform.
#[cfg(any(target_os = "macos", target_os = "ios"))]
pub const PLATFORM_FILE_EXTENSION: &str = "dylib";
///Dynamic link library file extension specific to the platform.
#[cfg(all(unix, not(any(target_os = "macos", target_os = "ios"))))]
pub const PLATFORM_FILE_EXTENSION: &str = "so";
///Dynamic link library file extension specific to the platform.
#[cfg(windows)]
pub const PLATFORM_FILE_EXTENSION: &str = "dll";

/// Contract definition file template
pub const DEF_RS: &str = r##"
fn main() {
    let contract_def = <#contract_fqn as odra::contract_def::HasContractDef>::contract_def();
    let code = odra_#backend_name_backend::codegen::gen_contract(contract_def, "#contract_fqn".to_string());

    use std::fs::File;
    use std::io::prelude::*;
    let mut file = File::create("src/#contract_name_wasm.rs").unwrap();
    file.write_all(&code.to_string().into_bytes()).unwrap();
}
    "##;

/// Main.rs file template
pub const MAIN_RS: &str = r##"
fn main() {}
    "##;
