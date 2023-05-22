default:
    just --list

install:
    cargo install --path . --locked

prepare:
    rustup target add wasm32-unknown-unknown
    sudo apt install wabt

test-project-generation:
    rm -rf testproject
    cargo odra new --name testproject --source release/0.3.0
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
