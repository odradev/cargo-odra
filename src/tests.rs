use crate::{builder, Build, Test};
use std::os::unix::process::CommandExt;
use std::process::Command;

pub(crate) fn test(test: &Test) {
    match test.backend {
        None => {
            test_mock_vm(test);
        }
        Some(_) => {
            test_backend(test);
        }
    }
}

fn test_backend(test: &Test) {
    builder::build(Build {
        backend: test.backend.clone(),
        repo_uri: test.repo_uri.clone(),
    });

    let mut test_args = get_test_args(test);
    test_args.append(&mut vec!["--no-default-features", "--features=wasm-test"]);

    println!("Running cargo test...");
    Command::new("cargo").args(test_args).exec();
}

fn test_mock_vm(test: &Test) {
    let test_args = get_test_args(test);

    Command::new("cargo").args(test_args).exec();
}

fn get_test_args(test: &Test) -> Vec<&str> {
    let mut test_args = vec!["test"];
    match &test.passthrough {
        None => {}
        Some(passthrough) => {
            let passthrough = passthrough.first().unwrap().as_os_str().to_str().unwrap();
            let mut vec: Vec<&str> = passthrough.split(' ').collect();
            test_args.append(&mut vec);
        }
    }
    test_args
}
