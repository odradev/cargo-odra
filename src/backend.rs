use std::fs;
use std::process::Command;

pub(crate) fn pull_backend(repo_uri: &String) {
    println!("Downloading repository from {}...", repo_uri);
    Command::new("git")
        .args(vec!["clone", repo_uri.as_str(), ".backend"])
        .output()
        .unwrap();
}

pub(crate) fn build_backend(backend: &String) {
    println!("Building {} backend...", backend);
    Command::new("cargo")
        .current_dir(".backend/test_env")
        .args(vec!["build"])
        .output()
        .expect("Couldn't build backend");
    println!("Copying lib...");
    fs::create_dir_all("./target/debug").unwrap();
    Command::new("cp")
        .args(vec![
            ".backend/test_env/target/debug/libodra_test_env.so",
            "./target/debug/libodra_test_env.so",
        ])
        .output()
        .expect("Couldn't copy lib");
}
