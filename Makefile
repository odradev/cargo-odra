default:
	cargo build

install:
	cargo install cargo-generate
	cargo build --release
	cp target/release/cargo-odra ~/.cargo/bin/cargo-odra