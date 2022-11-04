//! Errors
use crate::log;
use std::fmt::{Display, Formatter};
use std::path::PathBuf;
use std::process::exit;

// TODO: Use thiserror.
/// Errors enum
pub enum Error {
    CommandFailed(String),
    InvalidInternalCommand(String),
    FailedToReadCargo(String),
    WasmTargetNotInstalled,
    NotAnOdraProject,
    WasmstripNotInstalled,
    CurrentDirIsNotEmpty,
    FileAlreadyExists(PathBuf),
    ContractAlreadyInOdraToml(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            Error::CommandFailed(msg) => msg.to_string(),
            Error::InvalidInternalCommand(command) => format!("Invalid command {}", command),
            Error::FailedToReadCargo(error) => {
                format!("Failed to read Cargo.toml: {}", error)
            }
            Error::WasmTargetNotInstalled => {
                "wasm32-unknown-unknown target is not present, install it by executing:\n\
            rustup target add wasm32-unknown-unknown"
                    .to_string()
            }
            Error::NotAnOdraProject => {
                "This command can be executed only in folder with Odra project.".to_string()
            }
            Error::WasmstripNotInstalled => {
                "There was an error while running wasm-strip - is it installed?".to_string()
            }
            Error::CurrentDirIsNotEmpty => "Current directory is not empty.".to_string(),
            Error::FileAlreadyExists(path) => {
                format!("File {} already exists,", path.to_string_lossy())
            }
            Error::ContractAlreadyInOdraToml(name) => {
                format!("Contract {} already in Odra.toml", name)
            }
        };

        write!(f, "{}", msg)
    }
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
        log::error(&format!("{}", self));
        exit(self.code());
    }
}
