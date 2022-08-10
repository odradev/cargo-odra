//! Module implementing functions used by `cargo odra update` command
use crate::odra_toml::OdraConf;
use crate::{command, Backend, UpdateCommand};
use prettycli::{error, info};
use std::process::exit;

/// Runs `cargo update` on project and backends in .builder* folders. If backend is specified,
/// update will be made only in its folder
pub fn update(update_command: UpdateCommand) {
    if update_command.backend.is_some() {
        let backends = OdraConf::load().backends.unwrap();
        let backend = backends
            .get(&update_command.backend.unwrap())
            .unwrap_or_else(|| {
                error("No such backend");
                exit(1);
            });
        update_builder(backend);
    } else {
        update_everything();
    }
}

fn update_builder(backend: &Backend) {
    info(&format!(
        "Running cargo update for {} builder...",
        backend.name()
    ));
    command::cargo(backend.builder_path(), vec!["update"]);
}

fn update_project() {
    info("Running cargo update for project...");
    command::cargo(".".to_string(), vec!["update"]);
}

fn update_everything() {
    update_project();
    let backends = OdraConf::load().backends;
    if let Some(backends_map) = backends {
        for (_, backend) in backends_map {
            update_builder(&backend);
        }
    }
}
