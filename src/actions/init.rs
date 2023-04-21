//! Module responsible for initializing an Odra project.

use std::path::PathBuf;

/// InitAction configuration.
#[derive(Clone)]
pub struct InitAction {
    pub project_name: String,
    pub generate: bool,
    pub init: bool,
    pub repo_uri: String,
    pub source: Option<String>,
    pub workspace: bool,
    pub template: String,
    pub current_dir: PathBuf,
}

/// InitAction implementation.
impl InitAction {}
