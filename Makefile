default:
	cargo build

install:
	cargo build --release
	cp target/release/cargo-odra ~/.cargo/bin/cargo-odra