DEVELOPMENT_ODRA_BRANCH := "release/0.9.0"
BINARYEN_VERSION := "version_116"
BINARYEN_CHECKSUM := "c55b74f3109cdae97490faf089b0286d3bba926bb6ea5ed00c8c784fc53718fd"

default:
    just --list

install:
    cargo install --path . --locked

prepare:
    rustup target add wasm32-unknown-unknown
    sudo apt install wabt
    wget https://github.com/WebAssembly/binaryen/releases/download/{{BINARYEN_VERSION}}/binaryen-{{BINARYEN_VERSION}}-x86_64-linux.tar.gz || { echo "Download failed"; exit 1; }
    sha256sum binaryen-{{BINARYEN_VERSION}}-x86_64-linux.tar.gz | grep {{BINARYEN_CHECKSUM}} || { echo "Checksum verification failed"; exit 1; }
    tar -xzf binaryen-{{BINARYEN_VERSION}}-x86_64-linux.tar.gz || { echo "Extraction failed"; exit 1; }
    sudo cp binaryen-{{BINARYEN_VERSION}}/bin/wasm-opt /usr/local/bin/wasm-opt

test-project-generation-on-stable-odra:
    rm -rf testproject
    cargo odra new --name testproject
    just test-testproject

test-project-generation-on-future-odra:
    rm -rf testproject
    cargo odra new --name testproject --source {{DEVELOPMENT_ODRA_BRANCH}}
    just test-testproject

test-workspace-generation-on-stable-odra:
    rm -rf testproject
    cargo odra new --name testproject --template workspace
    just test-workspace-project

test-workspace-generation-on-future-odra:
    rm -rf testproject
    cargo odra new --name testproject --template workspace --source {{DEVELOPMENT_ODRA_BRANCH}}
    just test-workspace-project

test-testproject:
    cd testproject && rustup target add wasm32-unknown-unknown
    cd testproject && cargo odra generate -c plascoin
    cd testproject && cargo odra test
    cd testproject && cargo odra test -b casper
    cd testproject && cargo odra clean

test-workspace-project:
    cd testproject && rustup target add wasm32-unknown-unknown
    cd testproject && cargo odra generate -c plascoin -m flipper
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
