extern crate core;

use async_trait::async_trait;
use cucumber::{given, then, when, World, WorldInit};
use rand::Rng;
use std::convert::Infallible;
use std::path::Path;
use std::process::Command;

#[derive(Debug, WorldInit, Clone)]
pub struct CargoOdraWorld {
    folder: String,
    return_code: i32,
    stdout: Option<String>,
    stderr: Option<String>,
}

#[async_trait(?Send)]
impl World for CargoOdraWorld {
    type Error = Infallible;

    async fn new() -> Result<Self, Infallible> {
        let mut rng = rand::thread_rng();
        let folder = "test_run_".to_string() + &rng.gen::<u64>().to_string();
        std::fs::create_dir_all(folder.clone()).expect("Couldn't create temporary directory");
        Ok(Self {
            folder,
            return_code: 0,
            stdout: None,
            stderr: None,
        })
    }
}

#[given("clean workspace")]
fn cargo_odra(_world: &mut CargoOdraWorld) {}

#[given(expr = "odra set up")]
fn odra_set_up(world: &mut CargoOdraWorld) {
    run_command(world, format!("cargo odra init -n {}", world.folder));
}

#[when(expr = "I create a new folder called {word}")]
fn create_folder(world: &mut CargoOdraWorld, folder: String) {
    let folder = world.folder.clone() + "/" + &folder;
    std::fs::create_dir_all(folder).unwrap();
}

#[when(expr = "I run {string}")]
fn run_command(world: &mut CargoOdraWorld, command: String) {
    let mut split_command: Vec<&str> = command.split(' ').collect();
    let program = *split_command.first().unwrap();
    let args: Vec<&str> = split_command.drain(1..).collect();

    let folder = world.folder.clone();

    let output = Command::new(program)
        .current_dir(folder.clone())
        .args(args)
        .output()
        .expect(format!("failed to execute process {}", folder).as_str());

    world.stdout = Some(
        std::str::from_utf8(output.stdout.as_slice())
            .unwrap()
            .to_string(),
    );
    world.stderr = Some(
        std::str::from_utf8(output.stderr.as_slice())
            .unwrap()
            .to_string(),
    );

    world.return_code = output.status.code().unwrap();
}

#[then(expr = "folder named {word} exists")]
fn folder_exists(world: &mut CargoOdraWorld, folder_name: String) {
    let folder_name = world.folder.clone() + "/" + &folder_name;
    assert!(Path::new(&folder_name).exists());
}

#[then(expr = "I see {string}")]
fn output_contains(world: &mut CargoOdraWorld, contains: String) {
    let output = world.stdout.clone().unwrap() + &world.stderr.clone().unwrap();
    assert!(output.contains(&contains));
}

#[then(expr = "error code is {int}")]
fn exit_code(world: &mut CargoOdraWorld, error_code: i32) {
    assert_eq!(world.return_code, error_code);
}

#[tokio::main]
async fn main() {
    CargoOdraWorld::cucumber()
        .after(|_feature, _rule, _scenario, world| {
            Box::pin(async move {
                std::fs::remove_dir_all(world.unwrap().folder.clone()).unwrap();
            })
        })
        .run_and_exit("tests/features/")
        .await;
}
