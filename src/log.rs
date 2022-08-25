//! Logging functions

/// Info message
pub fn info(message: &str) {
    prettycli::info(message);
}

/// Warning message, not used yet - remove underscore when in use
pub fn _warn(message: &str) {
    prettycli::warn(message);
}

/// Error message
pub fn error(message: &str) {
    prettycli::error(message);
}

/// Critical message, not used yet - remove underscore when in use
pub fn _crit(message: &str) {
    prettycli::critical(message);
}
