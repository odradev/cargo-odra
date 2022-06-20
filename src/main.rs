use std::os::unix::process::CommandExt;
use std::process::Command;
use clap::{Parser, Subcommand};


#[derive(Parser)]
#[clap(name = "cargo")]
#[clap(bin_name = "cargo")]
enum Cargo {
    Odra(Odra),
}

#[derive(clap::Args)]
#[clap(author, version, about, long_about = None)]
struct Odra {
    #[clap(subcommand)]
    subcommand: OdraSubcommand,
}

#[derive(Subcommand)]
enum OdraSubcommand {
    New(New),
    Build(Build),
    Test(Test),
    #[clap(subcommand)]
    Generate(Generate),
}

#[derive(clap::Args)]
struct New {

}

#[derive(clap::Args)]
struct Init {

}

#[derive(clap::Args)]
struct Build {

}

#[derive(clap::Args)]
struct Test {

}

#[derive(clap::Subcommand)]
enum Generate {
    Contract(Contract),
}

#[derive(clap::Args)]
struct Contract {

}

fn main() {
    let Cargo::Odra(args) = Cargo::parse();
    match args.subcommand {
        OdraSubcommand::Build(_) => {
            println!("Build!");
        }
        OdraSubcommand::Test(_) => {
            println!("Test!");
        }
        OdraSubcommand::Generate(generate) => {
            match generate {
                Generate::Contract(_) => {
                    println!("Contract!");
                }
            }
        }
        OdraSubcommand::New(_) => {
            Command::new("cargo")
                .args(["generate", "odradev/odra-template"])
                .exec();
        }
    }
}