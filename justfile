DEVELOPMENT_ODRA_BRANCH := "release/0.7.1"

default:
    just --list

install:
    cargo install --path . --locked

prepare:
    rustup target add wasm32-unknown-unknown
    sudo apt install wabt

test-project-generation-on-stable-odra:
    rm -rf testproject
    cargo odra new --name testproject
    just test-testproject

test-project-generation-on-future-odra:
    rm -rf testproject
    cargo odra new --name testproject --source {{DEVELOPMENT_ODRA_BRANCH}}
    just test-testproject

test-testproject:
    cd testproject && rustup target add wasm32-unknown-unknown
    cd testproject && rustup component add rustfmt --toolchain nightly-2023-03-01-x86_64-unknown-linux-gnu
    cd testproject && cargo odra generate -c plascoin
    cd testproject && cargo odra test
    cd testproject && cargo odra test -b casper
    cd testproject && cargo odra clean

clippy:
	cargo clippy --all-targets -- -D warnings

check-lint: clippy
	cargo fmt -- --check

lint: clippy
	cargo fmt

clean:
	cargo clean
