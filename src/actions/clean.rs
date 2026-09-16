//! Module responsible for cleaning Odra projects.

use crate::{command, project::Project};

/// Removes wasm folders, and runs `cargo clean`.
pub fn clean_action(project: &Project) {
    project.workspace_members.iter().for_each(|member_root| {
        command::rm_dir(member_root.join("wasm"));
    });

    command::rm_dir(project.project_root().join("wasm"));

    command::cargo_clean(project.project_root());
}
