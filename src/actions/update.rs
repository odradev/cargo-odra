//! Module implementing functions used by `cargo odra update` command

use crate::{command, log, UpdateCommand};
use std::path::PathBuf;

/// Runs `cargo update` on project and backends in .builder* folders.
/// If backend is specified update will be made only in its folder
pub fn update_action(update_command: UpdateCommand) {
    if let Some(backend) = update_command.backend {
        update_builder_by_name(backend);
    } else {
        update_all_builders();
        update_project();
    }
}

fn update_builder_by_name(backend: String) {
    update_builder(PathBuf::from(format!(".builder_{}", backend)));
}

fn update_builder(builder: PathBuf) {
    log::info(format!(
        "Running cargo update for {} builder...",
        builder.to_str().unwrap()
    ));
    command::cargo(builder.to_str().unwrap().to_string(), vec!["update"]);
}

fn update_all_builders() {
    for builder_dir in glob::glob(".builder_*").unwrap().flatten() {
        update_builder(builder_dir);
    }
}

fn update_project() {
    log::info("Running cargo update for project...");
    command::cargo(".".to_string(), vec!["update"]);
}
