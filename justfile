DEVELOPMENT_ODRA_BRANCH := "release/3.0.0"
# Must stay >= 121: contracts are optimised with --llvm-memory-copy-fill-lowering,
# which older binaryen releases reject.
BINARYEN_VERSION := "version_125"
BINARYEN_CHECKSUM := "7c3bc16599c8274a04d34a504fe4be2047884f900e0e2da2f6fb9cd667183be4"

default:
    just --list

install:
    cargo install --path . --locked

prepare:
    rustup target add wasm32-unknown-unknown
    rustup toolchain install nightly
    rustup component add --toolchain nightly clippy
    rustup component add --toolchain nightly rustfmt
    sudo apt-get update
    sudo apt-get install -y wabt
    wget https://github.com/WebAssembly/binaryen/releases/download/{{ BINARYEN_VERSION }}/binaryen-{{ BINARYEN_VERSION }}-x86_64-linux.tar.gz || { echo "Download failed"; exit 1; }
    sha256sum binaryen-{{ BINARYEN_VERSION }}-x86_64-linux.tar.gz | grep {{ BINARYEN_CHECKSUM }} || { echo "Checksum verification failed"; exit 1; }
    tar -xzf binaryen-{{ BINARYEN_VERSION }}-x86_64-linux.tar.gz || { echo "Extraction failed"; exit 1; }
    sudo cp binaryen-{{ BINARYEN_VERSION }}/bin/wasm-opt /usr/local/bin/wasm-opt

# Project templates that differ in structure: single crate with a contract, empty single
# crate, a workspace with two contract crates and a CLI crate, and the CEP-18 / CEP-95
# token templates that pull in odra-modules.
TEMPLATES := "full blank workspace cep18 cep95"

# Generates a project from `template` against the latest Odra release (`stable`) or
# DEVELOPMENT_ODRA_BRANCH (`future`), then puts it through the cargo-odra commands:
# generate, build, schema, test on both backends, clean.
test-template template source="future":
    rm -rf testproject
    cargo odra new --name testproject --template {{ template }} {{ if source == "stable" { "" } else { "--source " + DEVELOPMENT_ODRA_BRANCH } }}
    just _exercise-testproject {{ template }}

# `test-template` for every template, against one Odra source.
test-all-templates source="future":
    for template in {{ TEMPLATES }}; do just test-template $template {{ source }}; done

# Adds a contract, builds, generates schemas, tests on both backends.
_exercise-testproject template:
    cd testproject && rustup target add wasm32-unknown-unknown
    cd testproject && cargo odra generate -c plascoin {{ if template == "workspace" { "-m flipper" } else { "" } }}
    {{ if template == "full" { "just test-contract-name-flexibility testproject src/plascoin.rs" } else if template == "workspace" { "just test-contract-name-flexibility testproject flipper/src/plascoin.rs" } else { "true" } }}
    cd testproject && cargo odra build
    cd testproject && cargo odra schema
    cd testproject && ls resources/casper_contract_schemas/*.json
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

# Run cargo-odra's own unit tests.
test:
    cargo test

# Audit dependencies for advisories, licenses, banned crates and sources.
# Install with `cargo install cargo-deny --locked`.
check-deny:
    cargo deny check

# Everything the CI pipeline runs, in the same order, against the current checkout.
# `just prepare` is left out: it needs sudo and changes the machine. Run it once by hand.
# Note that `install` replaces the cargo-odra on your PATH with the one from this checkout,
# which is what makes the generation recipes below test your branch.
ci: check-lint test check-deny install
    just test-all-templates future
    just test-all-templates stable

# The CI pipeline inside the GitHub Actions runner image, via nektos/act.
# Needs docker and act (https://nektosact.com). Slower than `just ci` and it always starts
# from a cold cargo cache, but it catches anything specific to the runner image.
# Pass act flags through, e.g. `just ci-act -j build_and_test` for a single job.
# Note: the `deny` job builds a container image, so it needs DNS to work inside
# `docker build`, which some local Docker setups do not have.
ci-act *ARGS:
    act push -W .github/workflows/ci-cargo-odra.yml -P ubuntu-latest=catthehacker/ubuntu:act-latest {{ ARGS }}

clippy:
    cargo +nightly clippy --all-targets -- -D warnings

check-lint: clippy
    cargo +nightly fmt -- --check

lint: clippy
    cargo +nightly fmt

clean:
    cargo clean
