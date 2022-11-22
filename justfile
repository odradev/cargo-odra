default:
    just --list

install:
    cargo install --path .

prepare-test-env:
    rustup target add wasm32-unknown-unknown
    sudo apt install wabt

test-project-generation:
    rm -rf testproject
    cargo odra new --name testproject --git-branch 0.2.0
    cd testproject && cargo odra generate -c plascoin
    cd testproject && cargo odra test
    cd testproject && cargo odra test -b casper
    cd testproject && cargo odra clean

clippy:
	cargo clippy --all-targets -- -D warnings

check-lint: clippy
	cargo +nightly fmt -- --check

lint: clippy
	cargo +nightly fmt

clean:
	cargo clean
