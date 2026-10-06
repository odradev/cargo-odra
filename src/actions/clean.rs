//! Module responsible for cleaning Odra projects.

use crate::{command, project::Project};

/// Removes wasm folders, and runs `cargo clean`.
///
/// Every workspace member is visited, not only the project root: `cargo-odra` 0.1.x copied the
/// built wasm into the member that defines a contract, and those copies would otherwise be left
/// behind, where the Casper test VM would find them before the freshly built ones at the root.
pub fn clean_action(project: &Project) {
    project.workspace_members.iter().for_each(|member_root| {
        command::rm_dir(member_root.join("wasm"));
    });

    command::rm_dir(project.project_root().join("wasm"));

    command::cargo_clean(project.project_root());
}
