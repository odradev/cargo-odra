pub const ODRA_CRATE_NAME: &str = "odra";

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
