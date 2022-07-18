default:
	cargo build --release

install:
	cargo build --release
	cp target/release/cargo-odra ~/.cargo/bin/cargo-odra