//! Module containing code that parses CLI input.

use std::env;

use clap::{Parser, Subcommand};

use crate::{
    actions::{build::BuildAction, clean::clean_action, init::InitAction, update::update_action},
    consts,
    project::Project,
};

#[derive(Parser)]
#[clap(name = "cargo")]
#[clap(bin_name = "cargo")]
/// Main command `cargo`.
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
    /// Be verbose.
    pub verbose: bool,

    #[clap(value_parser, long, short, global = true)]
    /// Be quiet, show only errors.
    pub quiet: bool,
}

#[derive(Subcommand)]
/// Subcommands of `cargo odra`.
pub enum OdraSubcommand {
    /// Creates a new Odra project.
    New(InitCommand),
    /// Initializes a new Odra project in an existing, empty directory.
    Init(InitCommand),
    /// Builds the project, including backend and producing wasm files.
    Build(BuildCommand),
    /// Runs test. Without the backend parameter, the tests will be run against Mock VM.
    Test(TestCommand),
    /// Generates boilerplate code for contracts.
    Generate(GenerateCommand),
    /// Cleans all temporary data generated by cargo odra.
    Clean(CleanCommand),
    /// Updates project alongside builders.
    Update(UpdateCommand),
}

#[derive(clap::Args)]
/// `cargo odra init`
pub struct InitCommand {
    /// Name which will be used as a name for the crate.
    #[clap(value_parser, long, short)]
    pub name: String,
    /// URI of the repository containing the template.
    #[clap(value_parser, long, short, default_value = consts::ODRA_TEMPLATE_GH_REPO)]
    pub repo_uri: String,
    /// Git branch to use.
    #[clap(value_parser, long, short, default_value = consts::ODRA_TEMPLATE_GH_BRANCH)]
    pub git_branch: String,
    /// Template to use. Default is "full", which contains a sample contract and a test.
    /// To see all available templates, run `cargo odra new --list`.
    #[clap(value_parser, long, short, default_value = consts::ODRA_TEMPLATE_DEFAULT_TEMPLATE)]
    pub template: String,
}

#[derive(clap::Args)]
/// `cargo odra build`
pub struct BuildCommand {
    /// Name of the backend that will be used for the build process (e.g. casper).
    #[clap(value_parser, long, short, possible_values = [consts::ODRA_CASPER_BACKEND])]
    pub backend: String,
}

#[derive(clap::Args, Debug)]
/// `cargo odra test`
pub struct TestCommand {
    /// If set, runs tests against a backend VM with the given name (e.g. casper).
    #[clap(value_parser, long, short, possible_values = [consts::ODRA_CASPER_BACKEND])]
    pub backend: Option<String>,
    /// A list of arguments is passed to the cargo test command.
    #[clap(raw = true)]
    pub args: Vec<String>,
    /// Skip building wasm files.
    #[clap(value_parser, long, short, default_value = "false")]
    pub skip_build: bool,
}

#[derive(clap::Args, Debug)]
/// `cargo odra generate`
pub struct GenerateCommand {
    /// Name of the contract to be created.
    #[clap(value_parser, long, short)]
    pub contract_name: String,
    /// Name of the module in which the contract will be created.
    #[clap(value_parser, long, short)]
    pub module: Option<String>,
}

#[derive(clap::Args, Debug)]
/// `cargo odra clean`
pub struct CleanCommand {}

#[derive(clap::Args, Debug)]
/// `cargo odra update`
pub struct UpdateCommand {
    /// If set, runs cargo update for the given builder instead of everyone.
    #[clap(value_parser, long, short, possible_values = [consts::ODRA_CASPER_BACKEND])]
    pub backend: Option<String>,
}

/// Cargo odra main parser function.
pub fn make_action() {
    let Cargo::Odra(args) = Cargo::parse();
    match args.subcommand {
        OdraSubcommand::Build(build) => {
            BuildAction::new(
                Project::detect(Some(env::current_dir().unwrap())),
                build.backend,
            )
            .build();
        }
        OdraSubcommand::Test(test) => {
            Project::detect(Some(env::current_dir().unwrap())).test(test);
        }
        OdraSubcommand::Generate(generate) => {
            Project::detect(Some(env::current_dir().unwrap())).generate(generate);
        }
        OdraSubcommand::New(init) => {
            Project::new(InitAction {
                project_name: init.name,
                generate: true,
                init: false,
                repo_uri: init.repo_uri,
                branch: init.git_branch,
                workspace: false,
                template: init.template,
            });
        }
        OdraSubcommand::Init(init) => {
            Project::new(InitAction {
                project_name: init.name,
                generate: true,
                init: true,
                repo_uri: init.repo_uri,
                branch: init.git_branch,
                workspace: false,
                template: init.template,
            });
        }
        OdraSubcommand::Clean(_) => {
            let project = Project::detect(Some(env::current_dir().unwrap()));
            clean_action(project.project_root());
        }
        OdraSubcommand::Update(update) => {
            let project = Project::detect(Some(env::current_dir().unwrap()));
            update_action(update, project.project_root());
        }
    }
}
