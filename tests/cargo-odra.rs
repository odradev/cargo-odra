use async_trait::async_trait;
use cucumber::{given, then, when, World, WorldInit};
use std::convert::Infallible;
use std::path;
use std::path::Path;
use std::process::{Command, Output};
// `World` is your shared, likely mutable state.
#[derive(Debug, WorldInit, Clone)]
pub struct CargoOdraWorld {
    return_code: i32,
    stdout: Option<String>,
    stderr: Option<String>,
}

// `World` needs to be implemented, so Cucumber knows how to construct it
// for each scenario.
#[async_trait(?Send)]
impl World for CargoOdraWorld {
    // We do require some error type.
    type Error = Infallible;
    async fn new() -> Result<Self, Infallible> {
        Ok(Self {
            return_code: 0,
            stdout: None,
            stderr: None,
        })
    }
}

// Steps are defined with `given`, `when` and `then` attributes.
#[given("clean workspace")]
fn cargo_odra(_world: &mut CargoOdraWorld) {
    let path = path::PathBuf::from("project");
    if path.exists() {
        std::fs::remove_dir_all(path).unwrap();
    }
}

#[given(expr = "odra set up in {word} folder")]
fn odra_set_up_in_folder(world: &mut CargoOdraWorld, folder: String) {
    let path = path::PathBuf::from(folder);
    if path.exists() {
        std::fs::remove_dir_all(path).unwrap();
    }
    run_command(world, "cargo odra new -n project".to_string());
}

#[when(expr = "I create a new folder called {word}")]
fn create_folder(_world: &mut CargoOdraWorld, folder: String) {
    std::fs::create_dir_all(folder).unwrap();
}

#[when(expr = "I run {string}")]
fn run_command(world: &mut CargoOdraWorld, command: String) {
    let output = command_result(command, None);
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

#[when(expr = "I run {string} in {word} folder")]
fn run_command_in_folder(world: &mut CargoOdraWorld, command: String, folder: String) {
    let output = command_result(command, Some(folder));
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

fn command_result(command: String, folder: Option<String>) -> Output {
    let mut split_command: Vec<&str> = command.split(' ').collect();
    let program = *split_command.first().unwrap();
    let args: Vec<&str> = split_command.drain(1..).collect();

    let folder = if folder.is_some() {
        folder.unwrap()
    } else {
        ".".to_string()
    };

    Command::new(program)
        .current_dir(folder)
        .args(args)
        .output()
        .expect("failed to execute process")
}

#[then(expr = "folder named {word} exists")]
fn folder_exists(_world: &mut CargoOdraWorld, folder_name: String) {
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

// This runs before everything else, so you can setup things here.
fn main() {
    futures::executor::block_on(CargoOdraWorld::run("tests/features/"));
}
