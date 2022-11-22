//! Logging functions

/// Info message.
pub fn info<T: AsRef<str>>(message: T) {
    prettycli::info(message.as_ref());
}

/// Warning message, not used yet - remove underscore when in use.
pub fn _warn<T: AsRef<str>>(message: T) {
    prettycli::warn(message.as_ref());
}

/// Error message.
pub fn error<T: AsRef<str>>(message: T) {
    prettycli::error(message.as_ref());
}

/// Critical message, not used yet - remove underscore when in use.
pub fn _crit<T: AsRef<str>>(message: T) {
    prettycli::critical(message.as_ref());
}
