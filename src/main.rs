//! Cargo Odra is a tool that helps you creating, maintaining, testing and building smart contracts
//! developed using the Odra framework.
//!
//! To see examples on how to use cargo odra, visit project's
//! [Github Page](https://github.com/odradev/cargo-odra).

use clap::{Parser, Subcommand};

mod actions;
mod cargo_toml;
mod command;
mod consts;
mod errors;
mod log;
mod odra_toml;
mod paths;

use actions::{
    build::BuildAction, clean::clean_action, generate::GenerateAction, init::InitAction,
    test::TestAction, update::update_action,
};

pub use command::command_output;

#[derive(Parser)]
#[clap(name = "cargo")]
#[clap(bin_name = "cargo")]
/// Main command `cargo`
pub enum Cargo {
    Odra(Odra),
}

#[derive(clap::Args)]
#[clap(author, version, about, long_about = None)]
/// `cargo odra`
pub struct Odra {
    #[clap(subcommand)]
    subcommand: OdraSubcommand,

    #[clap(value_parser, long, short, global = true)]
    /// Be verbose
    verbose: bool,

    #[clap(value_parser, long, short, global = true)]
    /// Be quiet, show only errors
    quiet: bool,
}

#[derive(Subcommand)]
/// Subcommands of `cargo odra`
pub enum OdraSubcommand {
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
    /// Updates project alongside builders
    Update(UpdateCommand),
}

#[derive(clap::Args)]
/// `cargo odra init`
pub struct InitCommand {
    /// Name which will be used as a name for the crate
    #[clap(value_parser, long, short)]
    name: String,
    /// URI of the repository containing the template
    #[clap(value_parser, long, short, default_value = consts::ODRA_TEMPLATE_GH_REPO)]
    repo_uri: String,
}

#[derive(clap::Args)]
///  `cargo odra build`
pub struct BuildCommand {
    /// Name of the backend that will be used for the build process (e.g. casper)
    #[clap(value_parser, long, short, possible_values = [consts::ODRA_CASPER_BACKEND])]
    backend: String,
}

#[derive(clap::Args, Debug)]
///  `cargo odra test`
pub struct TestCommand {
    /// If set, tests will be run against a backend VM with given name (e.g. casper)
    #[clap(value_parser, long, short, possible_values = [consts::ODRA_CASPER_BACKEND])]
    backend: Option<String>,
    /// A list of arguments that will be passed to the cargo test command
    #[clap(raw = true)]
    args: Vec<String>,
}

#[derive(clap::Args, Debug)]
///  `cargo odra generate`
pub struct GenerateCommand {
    /// Name of the contract to be created
    #[clap(value_parser, long, short)]
    contract_name: String,
}

#[derive(clap::Args, Debug)]
///  `cargo odra clean`
pub struct CleanCommand {}

#[derive(clap::Args, Debug)]
///  `cargo odra update`
pub struct UpdateCommand {
    /// If set, update will be run for given builder, instead of everyone
    #[clap(value_parser, long, short, possible_values = [consts::ODRA_CASPER_BACKEND])]
    backend: Option<String>,
}

/// Cargo odra main function
fn main() {
    let Cargo::Odra(args) = Cargo::parse();
    match args.subcommand {
        OdraSubcommand::Build(build) => {
            BuildAction::new(build.backend).build();
        }
        OdraSubcommand::Test(test) => {
            TestAction::new(test).test();
        }
        OdraSubcommand::Generate(generate) => {
            GenerateAction::new(generate).generate_contract();
        }
        OdraSubcommand::New(init) => {
            InitAction::new(init).generate_project(false);
        }
        OdraSubcommand::Init(init) => {
            InitAction::new(init).generate_project(true);
        }
        OdraSubcommand::Clean(_) => {
            clean_action();
        }
        OdraSubcommand::Update(update) => update_action(update),
    }
}
