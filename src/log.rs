//! Logging functions

/// Info message
pub fn info(message: &str) {
    prettycli::info(message);
}

/// Warning message
pub fn warn(message: &str) {
    prettycli::warn(message);
}

/// Error message
pub fn error(message: &str) {
    prettycli::error(message);
}

/// Critical message
pub fn _crit(message: &str) {
    prettycli::critical(message);
}
