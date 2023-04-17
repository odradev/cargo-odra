//! Errors.

use std::{path::PathBuf, process::exit};

use crate::log;

/// Errors enum.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Command {0} failed.")]
    CommandFailed(String),

    #[error("Invalid command {0}.")]
    InvalidInternalCommand(String),

    #[error("Failed to read Cargo.toml: {0}.")]
    FailedToReadCargo(String),

    #[error("wasm32-unknown-unknown target is not present, install it by executing:\n\rustup target add wasm32-unknown-unknown")]
    WasmTargetNotInstalled,

    #[error("This command can be executed only in folder with Odra project.")]
    NotAnOdraProject,

    #[error("There was an error while running wasm-strip - is it installed?")]
    WasmstripNotInstalled,

    #[error("Current directory is not empty.")]
    CurrentDirIsNotEmpty,

    #[error("File {0} already exists.")]
    FileAlreadyExists(PathBuf),

    #[error("Contract {0} already in Odra.toml")]
    ContractAlreadyInOdraToml(String),

    #[error("Removing {0} directory failed.")]
    RemoveDirNotPossible(PathBuf),

    #[error("Module {0} not found.")]
    ModuleNotFound(String),

    #[error("Odra.toml not found at location {0}")]
    OdraTomlNotFound(PathBuf),

    #[error("Not implemented: {0}")]
    NotImplemented(String),
}

impl Error {
    /// Returns error code.
    pub fn code(&self) -> i32 {
        match self {
            Error::CommandFailed(_) => 1,
            Error::InvalidInternalCommand(_) => 2,
            Error::FailedToReadCargo(_) => 3,
            Error::WasmTargetNotInstalled => 4,
            Error::NotAnOdraProject => 5,
            Error::WasmstripNotInstalled => 6,
            Error::CurrentDirIsNotEmpty => 7,
            Error::FileAlreadyExists(_) => 8,
            Error::ContractAlreadyInOdraToml(_) => 9,
            Error::RemoveDirNotPossible(_) => 10,
            Error::ModuleNotFound(_) => 11,
            Error::OdraTomlNotFound(_) => 12,
            Error::NotImplemented(_) => 13,
        }
    }

    /// Logs error message and exits with the given error code.
    pub fn print_and_die(&self) -> ! {
        log::error(self.to_string());
        exit(self.code());
    }
}
