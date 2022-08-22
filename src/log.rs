use log::{error, info, warn};

pub fn info(message: &str) {
    info!("{}", message);
    prettycli::info(message);
}

pub fn warn(message: &str) {
    warn!("{}", message);
    prettycli::warn(message);
}

pub fn error(message: &str) {
    error!("{}", message);
    prettycli::error(message);
}

pub fn crit(message: &str) {
    error!("{}", message);
    prettycli::critical(message);
}