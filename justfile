DEVELOPMENT_ODRA_BRANCH := "release/2.5.1"
BINARYEN_VERSION := "version_116"
BINARYEN_CHECKSUM := "c55b74f3109cdae97490faf089b0286d3bba926bb6ea5ed00c8c784fc53718fd"

default:
    just --list

install:
    cargo install --path . --locked

prepare:
    rustup target add wasm32-unknown-unknown
    rustup toolchain install nightly
    rustup component add --toolchain nightly-x86_64-unknown-linux-gnu clippy
    rustup component add --toolchain nightly-x86_64-unknown-linux-gnu rustfmt
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
    just test-contract-name-flexibility testproject src/plascoin.rs
    cd testproject && cargo odra test
    cd testproject && cargo odra test -b casper
    cd testproject && cargo odra clean

test-workspace-project:
    cd testproject && rustup target add wasm32-unknown-unknown
    cd testproject && cargo odra generate -c plascoin -m flipper
    just test-contract-name-flexibility testproject flipper/src/plascoin.rs
    cd testproject && cargo odra test
    cd testproject && cargo odra test -b casper
    cd testproject && cargo odra clean

# Verify -c flag accepts various case formats for contract names
test-contract-name-flexibility project_dir source_file:
    # Test with lowercase input against CamelCase contract (Plascoin)
    cd {{project_dir}} && cargo odra build -c plascoin
    # Test with snake_case input against CamelCase contract (Flipper)
    cd {{project_dir}} && cargo odra build -c flipper
    # Rename Plascoin to PLASCOIN and verify uppercased names work
    cd {{project_dir}} && sed 's/Plascoin/PLASCOIN/g' Odra.toml > Odra.toml.tmp && mv Odra.toml.tmp Odra.toml
    cd {{project_dir}} && sed 's/Plascoin/PLASCOIN/g' {{source_file}} > {{source_file}}.tmp && mv {{source_file}}.tmp {{source_file}}
    cd {{project_dir}} && cargo odra build -c PLASCOIN
    cd {{project_dir}} && cargo odra build -c plascoin
    # Revert rename
    cd {{project_dir}} && sed 's/PLASCOIN/Plascoin/g' Odra.toml > Odra.toml.tmp && mv Odra.toml.tmp Odra.toml
    cd {{project_dir}} && sed 's/PLASCOIN/Plascoin/g' {{source_file}} > {{source_file}}.tmp && mv {{source_file}}.tmp {{source_file}}

clippy:
	cargo +nightly clippy --all-targets -- -D warnings

check-lint: clippy
	cargo +nightly fmt -- --check

lint: clippy
	cargo +nightly fmt

clean:
	cargo clean
