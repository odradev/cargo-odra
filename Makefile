default:
	cargo build --release

install:
	cargo build --release
	cp target/release/cargo-odra ~/.cargo/bin/cargo-odra

test-project-generation:
	rm -rf test-project
	cargo odra new -n test-project
	cd test-project && cargo odra generate -c plascoin
	cd test-project && cargo odra test
	cd test-project && cargo odra test -b casper
