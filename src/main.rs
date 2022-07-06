mod backend;
mod builder;
mod cargo_toml;
mod generate;
mod init;
mod odra_toml;
mod tests;

use crate::builder::Builder;
use crate::generate::Generate;
use crate::init::Init;
use crate::tests::Tests;
use clap::{Parser, Subcommand};
use std::ffi::OsString;

#[derive(Parser)]
#[clap(name = "cargo")]
#[clap(bin_name = "cargo")]
enum Cargo {
    Odra(Odra),
}

/// Cargo Odra is a tool that helps you creating, maintaining, testing and building smart contracts
/// developed using the Odra framework
#[derive(clap::Args)]
#[clap(author, version, about, long_about = None)]
struct Odra {
    #[clap(subcommand)]
    subcommand: OdraSubcommand,
}

#[derive(Subcommand)]
enum OdraSubcommand {
    /// Creates a new Odra project
    New(InitCommand),
    /// Initializes a new Odra project in an existing, empty directory
    Init(InitCommand),
    /// Builds the project, including backend and producing wasm files
    Build(BuildCommand),
    /// Runs test. Without the backend parameter, the tests will be run against Mock VM
    Test(TestCommand),
    /// Helper which will generate boilerplate code for contracts
    Generate(GenerateCommand),
}

#[derive(clap::Args)]
pub struct InitCommand {
    /// Name which will be used as a name for the crate
    #[clap(value_parser, long, short)]
    name: Option<String>,
    /// URI of the repository containing the template
    #[clap(value_parser, long, short)]
    repo_uri: Option<String>,
}

#[derive(clap::Args)]
pub struct BuildCommand {
    /// Name of the backend that will be used for the build process (e.g. casper, near)
    #[clap(value_parser, long, short)]
    backend: Option<String>,
    /// URI of the repository containing the backend code
    #[clap(value_parser, long, short)]
    repo_uri: Option<String>,
}

#[derive(clap::Args, Debug)]
pub struct TestCommand {
    /// If set, tests will be run against a backend VM with given name (e.g. casper, near)
    #[clap(value_parser, long, short)]
    backend: Option<String>,
    /// URI of the repository containing the backend code
    #[clap(value_parser, long, short)]
    repo_uri: Option<String>,
    /// A list of parameters that will be passed to the cargo test command
    #[clap(value_parser, long, short)]
    passthrough: Option<Vec<OsString>>,
}

#[derive(clap::Args, Debug)]
pub struct GenerateCommand {
    /// Name of the contract to be created
    #[clap(value_parser, long, short)]
    contract_name: String,
}

fn main() {
    let Cargo::Odra(args) = Cargo::parse();
    match args.subcommand {
        OdraSubcommand::Build(build) => {
            Builder::new(build).build();
        }
        OdraSubcommand::Test(test) => {
            Tests::new(test).test();
        }
        OdraSubcommand::Generate(generate) => {
            Generate::new(generate).generate_contract();
        }
        OdraSubcommand::New(init) => {
            Init::new(init).new_project();
        }
        OdraSubcommand::Init(init) => {
            Init::new(init).init_project();
        }
    }
}
