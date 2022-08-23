use crate::error;
use std::fmt::{Display, Formatter};
use std::process::exit;

pub enum Error {
    CommandFailed(String),
    InvalidInternalCommand(String),
    FailedToReadCargo(String),
    NoBackendConfigured,
    NoSuchBackend,
    WasmTargetNotInstalled,
    NotAnOdraProject,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            Error::CommandFailed(msg) => msg.to_string(),
            Error::InvalidInternalCommand(command) => format!("Invalid command {}", command),
            Error::FailedToReadCargo(error) => {
                format!("Failed to read Cargo.toml: {}", error)
            }
            Error::NoBackendConfigured => "No backend configured".to_string(),
            Error::NoSuchBackend => "No such backend".to_string(),
            Error::WasmTargetNotInstalled => {
                "wasm32-unknown-unknown target is not present, install it by executing:\n\
            rustup target add wasm32-unknown-unknown"
                    .to_string()
            }
            Error::NotAnOdraProject => {
                "This command can be executed only in folder with Odra project.".to_string()
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
            Error::NoBackendConfigured => 4,
            Error::NoSuchBackend => 5,
            Error::WasmTargetNotInstalled => 6,
            Error::NotAnOdraProject => 7,
        }
    }

    pub fn print_and_die(&self) -> ! {
        error(&format!("{}", self));
        exit(self.code());
    }
}
