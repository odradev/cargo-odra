mod backend;
mod builder;
mod cargo_toml;
mod generate;
mod odra_toml;
mod tests;

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
    New(New),
    /// Initializes a new Odra project in an existing, empty directory
    Init(Init),
    /// Builds the project, including backend and producing wasm files
    Build(Build),
    /// Runs test. Without the backend parameter, the tests will be run against Mock VM
    Test(Test),
    /// Helper which will generate boilerplate code for contracts
    Generate(Generate),
}

#[derive(clap::Args)]
struct New {
    /// Name which will be used as a folder name and name for the crate
    #[clap(value_parser, long, short)]
    name: Option<String>,
}

#[derive(clap::Args)]
struct Init {
    /// Name which will be used as a name for the crate
    #[clap(value_parser, long, short)]
    name: Option<String>,
}

#[derive(clap::Args)]
struct Build {
    /// Name of the backend that will be used for the build process (e.g. casper, near)
    #[clap(value_parser, long, short)]
    backend: Option<String>,
}

#[derive(clap::Args, Debug)]
struct Test {
    /// If set, tests will be run against a backend VM with given name (e.g. casper, near)
    #[clap(value_parser, long, short)]
    backend: Option<String>,
    /// A list of parameters that will be passed to the cargo test command
    #[clap(value_parser, long, short)]
    passthrough: Option<Vec<OsString>>,
}

#[derive(clap::Args, Debug)]
struct Generate {
    /// Name of the contract to be created
    #[clap(value_parser, long, short)]
    contract_name: String,
}

fn main() {
    let Cargo::Odra(args) = Cargo::parse();
    match args.subcommand {
        OdraSubcommand::Build(build) => {
            builder::build(build.backend);
        }
        OdraSubcommand::Test(test) => {
            tests::test(&test);
        }
        OdraSubcommand::Generate(generate) => {
            generate::generate_contract(&generate);
        }
        OdraSubcommand::New(_) => {
            generate::new_project();
        }
        OdraSubcommand::Init(_) => {
            generate::init_project();
        }
    }
}
