default:
    just --list

install:
    cargo install --path . --locked

prepare:
    rustup target add wasm32-unknown-unknown
    sudo apt install wabt

clippy:
	cargo clippy --all-targets -- -D warnings

check-lint: clippy
	cargo fmt -- --check

lint: clippy
	cargo fmt

clean:
	cargo clean
