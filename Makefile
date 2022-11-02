default:
	cargo build --release

install:
	cargo build --release
	cp target/release/cargo-odra ~/.cargo/bin/cargo-odra
	sudo apt install wabt
	rustup target add wasm32-unknown-unknown

test:
	cargo test

test-project-generation:
	rm -rf testproject
	cargo odra new -n testproject
	cd testproject && cargo odra generate -c plascoin
	cd testproject && cargo odra test
	cd testproject && cargo odra backend add --package casper --name casper
	cd testproject && cargo odra test -b casper
