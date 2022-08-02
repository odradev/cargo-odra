mod backend;
mod cargo_toml;
mod clean;
mod command;
mod consts;
mod generate;
mod init;
mod odra_dependency;
mod odra_toml;
mod tests;

use crate::backend::Backend;
use crate::clean::Clean;
use crate::generate::Generate;
use crate::init::Init;
use crate::odra_toml::assert_odra_toml;
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
    /// Cleans all temporary data generated by cargo odra
    Clean(CleanCommand),
    /// Manages backends
    #[clap(subcommand)]
    Backend(BackendCommand),
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

#[derive(Subcommand)]
enum BackendCommand {
    Add(AddBackendCommand),
    Remove(RemoveBackendCommand),
    List(ListBackendsCommand),
}

#[derive(clap::Args)]
pub struct AddBackendCommand {
    /// Name of the backend package (e.g. casper)
    #[clap(value_parser, long, short)]
    package: String,
    /// Name of the backend that will be used for the build process (e.g. casper)
    #[clap(value_parser, long, short)]
    name: String,
    /// URI of the repository containing the backend code
    #[clap(value_parser, long, short)]
    repo_uri: Option<String>,
    /// Branch name
    #[clap(value_parser, long, short)]
    branch: Option<String>,
    /// Version of backend crate
    #[clap(value_parser, long, short)]
    version: Option<String>,
    /// Local path
    #[clap(value_parser, long, short)]
    path: Option<String>,
}

impl AddBackendCommand {
    pub fn path(&self) -> Option<String> {
        if self.path.is_none() {
            None
        } else {
            let path = self.path.clone().unwrap();
            if !path.ends_with('/') {
                return Some(format!("{}/", path));
            }
            Some(path)
        }
    }
}

#[derive(clap::Args)]
pub struct RemoveBackendCommand {
    /// Name of the backend that will be used for the build process (e.g. casper)
    #[clap(value_parser, long, short)]
    name: String,
}

#[derive(clap::Args)]
pub struct ListBackendsCommand {}

#[derive(clap::Args)]
pub struct BuildCommand {
    /// Name of the backend that will be used for the build process (e.g. casper)
    #[clap(value_parser, long, short)]
    backend: String,
    /// URI of the repository containing the backend code
    #[clap(value_parser, long, short)]
    repo_uri: Option<String>,
}

#[derive(clap::Args, Debug)]
pub struct TestCommand {
    /// If set, tests will be run against a backend VM with given name (e.g. casper)
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

#[derive(clap::Args, Debug)]
pub struct CleanCommand {}

fn main() {
    let Cargo::Odra(args) = Cargo::parse();
    match args.subcommand {
        OdraSubcommand::Build(build) => {
            assert_odra_toml();
            Backend::load(build.backend).build();
        }
        OdraSubcommand::Test(test) => {
            assert_odra_toml();
            Tests::new(test).test();
        }
        OdraSubcommand::Generate(generate) => {
            assert_odra_toml();
            Generate::new(generate).generate_contract();
        }
        OdraSubcommand::New(init) => {
            Init::new(init).generate_project(false);
        }
        OdraSubcommand::Init(init) => {
            Init::new(init).generate_project(true);
        }
        OdraSubcommand::Clean(_) => {
            assert_odra_toml();
            Clean::new().clean();
        }
        OdraSubcommand::Backend(backend) => match backend {
            BackendCommand::Add(add) => match Backend::add(add) {
                true => {
                    println!("Added.");
                }
                false => {
                    println!("Backend already exists.");
                }
            },
            BackendCommand::Remove(remove) => match Backend::remove(remove) {
                true => {
                    println!("Removed.");
                }
                false => {
                    println!("No such backend.");
                }
            },
            BackendCommand::List(_) => {
                todo!()
            }
        },
    }
}
