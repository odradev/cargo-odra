//! Errors
use crate::log;
use std::path::PathBuf;
use std::process::exit;

/// Errors enum
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
}

impl Error {
    pub fn code(&self) -> i32 {
        match self {
            Error::CommandFailed(_) => 1,
            Error::InvalidInternalCommand(_) => 2,
            Error::FailedToReadCargo(_) => 3,
            Error::WasmTargetNotInstalled => 6,
            Error::NotAnOdraProject => 7,
            Error::WasmstripNotInstalled => 8,
            Error::CurrentDirIsNotEmpty => 9,
            Error::FileAlreadyExists(_) => 10,
            Error::ContractAlreadyInOdraToml(_) => 11,
        }
    }

    pub fn print_and_die(&self) -> ! {
        log::error(self.to_string());
        exit(self.code());
    }
}
