//! Module implementing functions used by `cargo odra update` command.

use std::path::PathBuf;

use crate::{cli::UpdateCommand, command, log, paths};

/// Runs `cargo update` on project and backends in .builder* folders.
/// If backend is specified update will be made only in its folder.
pub fn update_action(update_command: UpdateCommand) {
    if let Some(backend) = update_command.backend {
        let builder_paths = paths::BuilderPaths::new(backend);
        update_builder(builder_paths.root());
    } else {
        update_all_builders();
        update_project();
    }
}

/// Update a builder crate.
fn update_builder(builder: PathBuf) {
    log::info(format!(
        "Running cargo update for {} builder...",
        builder.to_str().unwrap()
    ));
    command::cargo_update(builder);
}

/// Update all builders.
fn update_all_builders() {
    for builder_dir in glob::glob(".builder_*").unwrap().flatten() {
        update_builder(builder_dir);
    }
}

/// Update root project.
fn update_project() {
    log::info("Running cargo update for project...");
    command::cargo_update(PathBuf::from("."));
}
