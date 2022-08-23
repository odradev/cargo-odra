pub fn info(message: &str) {
    prettycli::info(message);
}

pub fn warn(message: &str) {
    prettycli::warn(message);
}

pub fn error(message: &str) {
    prettycli::error(message);
}

pub fn _crit(message: &str) {
    prettycli::critical(message);
}
