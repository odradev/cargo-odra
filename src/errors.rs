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

    #[error("wasm32-unknown-unknown target is not present, install it by executing:\nrustup target add wasm32-unknown-unknown")]
    WasmTargetNotInstalled,

    #[error("This command can be executed only in folder with Odra project.")]
    NotAnOdraProject,

    #[error("There was an error while running wasm-strip - is it installed?")]
    WasmstripDidNotFinish,

    #[error("There was an error while running wasm-opt - is it installed?")]
    WasmoptDidNotFinish,

    #[error("wasm-opt was not found. Install binaryen and make sure it is in your PATH:\nhttps://github.com/WebAssembly/binaryen/releases")]
    WasmoptNotInstalled,

    #[error("wasm-opt reports binaryen {0}, but {1} or newer is required.\nBuilding contracts needs --llvm-memory-copy-fill-lowering, which binaryen {0} does not support.\nPackages shipped by Linux distributions are usually too old - install a release from:\nhttps://github.com/WebAssembly/binaryen/releases")]
    WasmoptTooOld(u32, u32),

    #[error("wasm-strip was not found. Install wabt and make sure it is in your PATH:\nhttps://github.com/WebAssembly/wabt")]
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

    #[error("Failed to fetch template: {0}")]
    FailedToFetchTemplate(String),

    #[error("Failed to parse template: {0}")]
    FailedToParseTemplate(String),

    #[error("Could not determine the current directory, please make sure you have permissions to access it.")]
    CouldNotDetermineCurrentDirectory,

    #[error("Contract {0} not found in Odra.toml")]
    ContractNotFound(String),

    #[error("Contract {0} defined multiple times in Odra.toml, please make sure every contract has a unique name.")]
    ContractDuplicate(String),

    #[error("Odra is not a dependency of this project.")]
    OdraNotADependency,

    #[error("Failed to generate project from template: {0}")]
    FailedToGenerateProjectFromTemplate(String),

    #[error("Failed to parse the argument: {0}")]
    FailedToParseArgument(String),

    #[error("Malformed fqn of contract")]
    MalformedFqn,

    #[error("src/lib.rs not found.")]
    LibRsNotFound,

    #[error("Module {0} already in src/lib.rs")]
    ModuleAlreadyInLibRs(String),

    #[error("Project is a workspace, crate name is required")]
    CrateNotProvided,

    #[error("Crate for contract {0} not found in workspace members")]
    CrateOfContractNotFound(String),

    #[error("Failed to fetch templates file from {0}")]
    FailedToFetchTemplatesFile(String),

    #[error("Failed to parse templates file from {0}")]
    FailedToParseTemplatesFile(String),

    #[error("Template {0} not found in templates.json")]
    TemplateNotFound(String),

    #[error("Incorrect template type.")]
    IncorrectTemplateType,

    #[error("Client already exists in the project.")]
    ClientAlreadyExists,

    #[error("Couldn't create client.")]
    ClientCreateFailed,

    #[error("Project is not a workspace.")]
    NotWorkspace,

    #[error(
        "wasm-pack is not installed. Please install it from https://github.com/drager/wasm-pack"
    )]
    WasmPackNotInstalled,

    #[error("Toml serialization error: {0}")]
    TomlSerializationFailed(#[from] toml::ser::Error),

    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),
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
            Error::WasmstripDidNotFinish => 6,
            Error::CurrentDirIsNotEmpty => 7,
            Error::FileAlreadyExists(_) => 8,
            Error::ContractAlreadyInOdraToml(_) => 9,
            Error::RemoveDirNotPossible(_) => 10,
            Error::ModuleNotFound(_) => 11,
            Error::OdraTomlNotFound(_) => 12,
            Error::FailedToFetchTemplate(_) => 14,
            Error::FailedToParseTemplate(_) => 15,
            Error::CouldNotDetermineCurrentDirectory => 16,
            Error::ContractNotFound(_) => 17,
            Error::OdraNotADependency => 18,
            Error::FailedToGenerateProjectFromTemplate(_) => 19,
            Error::FailedToParseArgument(_) => 20,
            Error::MalformedFqn => 21,
            Error::LibRsNotFound => 22,
            Error::ModuleAlreadyInLibRs(_) => 23,
            Error::WasmoptDidNotFinish => 24,
            Error::CrateNotProvided => 25,
            Error::CrateOfContractNotFound(_) => 26,
            Error::FailedToFetchTemplatesFile(_) => 27,
            Error::FailedToParseTemplatesFile(_) => 28,
            Error::TemplateNotFound(_) => 29,
            Error::IncorrectTemplateType => 30,
            Error::ContractDuplicate(_) => 31,
            Error::ClientAlreadyExists => 32,
            Error::ClientCreateFailed => 33,
            Error::NotWorkspace => 34,
            Error::WasmPackNotInstalled => 35,
            Error::TomlSerializationFailed(_) => 36,
            Error::IO(_) => 37,
            Error::WasmoptNotInstalled => 38,
            Error::WasmoptTooOld(_, _) => 39,
            Error::WasmstripNotInstalled => 40,
        }
    }

    /// Logs error message and exits with the given error code.
    pub fn print_and_die(&self) -> ! {
        log::error(self.to_string());
        exit(self.code());
    }
}
