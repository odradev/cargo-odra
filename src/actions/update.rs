//! Module implementing functions used by `cargo odra update` command.

use std::path::PathBuf;

use crate::{cli::UpdateCommand, command, log};

/// Runs `cargo update` on project and backends in .builder* folders.
/// If backend is specified update will be made only in its folder.
pub fn update_action(_update_command: UpdateCommand, _project_root: PathBuf) {
    update_project();
}

/// Update root project.
fn update_project() {
    log::info("Running cargo update for project...");
    command::cargo_update(PathBuf::from("."));
}
