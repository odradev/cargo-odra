//! Module responsible for initializing an Odra project.

/// InitAction configuration.
#[derive(Clone)]
pub struct InitAction {
    pub project_name: String,
    pub generate: bool,
    pub init: bool,
    pub repo_uri: String,
    pub branch: String,
    pub workspace: bool,
}

/// InitAction implementation.
impl InitAction {}
