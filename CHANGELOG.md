# Changelog

Changelog for `cargo-odra`.

## [Unreleased]

### Fixed
- Contract crates that inherit fields from a workspace (`license = { workspace = true }` and the
  like) no longer fail with `not all fields of ... have been present in workspace.package`.
  The workspace root is now located the way Cargo does it: `package.workspace` first, otherwise
  the nearest ancestor `Cargo.toml` with a `[workspace]` section, skipping plain package
  manifests in between.

### Added
- `deny.toml`, `just check-deny` and a CI job auditing dependencies for security advisories,
  licenses, banned crates and unexpected sources.
- `just ci` runs the whole CI pipeline locally, in the same order, so a branch can be checked
  without waiting on GitHub. `just ci-act` runs the workflow itself in the GitHub runner image
  via [act](https://nektosact.com).

### Changed
- All dependencies upgraded. Seven RUSTSEC advisories came from `cargo-generate` 0.21 and
  `ureq` 2; both are on current majors and `rustls` is pinned to the patched 0.23.45. The one
  remaining advisory, unmaintained `smartstring` reached through `rhai` <- `cargo-generate`, has
  no published upgrade and is ignored in `deny.toml` with the reason recorded.
- `serde_json` is a direct dependency; `ureq` 3 no longer re-exports it. `colored` was dropped,
  nothing used it.
- Project generation against the latest Odra release is tested in CI again. It had been commented
  out since 1.4.0 in 2024. cargo-odra targets Odra 3.0 without dropping 2.x.
- `DEVELOPMENT_ODRA_BRANCH` in the justfile moved from `release/2.5.1` to `release/3.0.0`, so CI
  generates test projects against the Odra branch actually being developed.
- The justfile installs binaryen 125 instead of 116. Contracts are optimised with
  `--llvm-memory-copy-fill-lowering`, which binaryen below 121 rejects, so `just prepare` used to
  set up an environment in which no contract could be built.
- CI runs on `ubuntu-latest` instead of a BuildJet runner, with up-to-date actions
  (`actions/checkout@v4`, `dtolnay/rust-toolchain`, `extractions/setup-just@v2`) in place of
  `actions/checkout@v2` and the archived `actions-rs/toolchain`. Cargo's registry and git
  checkouts are cached, and a new run on the same branch cancels the previous one.
- CI now runs the unit tests (`just test`), which nothing executed before.
- `just prepare` no longer hardcodes the `x86_64-unknown-linux-gnu` host triple when adding
  nightly components, and installs `wabt` with `apt-get update` first and a non-interactive
  `-y`, so it works on a machine whose package lists are cold rather than only on a warm
  GitHub runner.

## [0.1.8] - 2026-08-04

### Fixed
- `wasm-opt` failures now say what is actually wrong. A missing `wasm-opt` and one too old for the
  `--llvm-memory-copy-fill-lowering` flag (binaryen < 121, which is what most Linux distributions
  package) both used to report `There was an error while running wasm-opt - is it installed?`.
  The version is now checked up front and reported, with a link to the binaryen releases.
- A missing `wasm-strip` is reported as missing rather than as a failed run.

## [0.1.7] - 2026-04-03

### Added
- Added `wasm-opt` flags.

### Fixed
- `-c` flag to accept contract names in any case format.


## [0.1.6] - 2025-06-09

### Added
- Support for `odra-cli`.

## [0.1.5] - 2025-06-06

### Added

- Support for `odra-cli` placeholder in `Odra.toml` to specify the CLI version
  used for the project.

## [0.1.4] - 2024-07-30

### Added

- Tests filter with `--test` flag.

## [0.1.3] - 2024-06-17

### Fixed

- Handle empty `Odra.toml`.

## [0.1.2] - 2024-05-23

### Added

- Support for defining template of a contract to grab when using `generate` command.
- Ability to list all available templates using `list-templates` command.

### Fixed

- Fixed error that caused contracts to fail to be built when coming from crates with
  hyphens in their name.

## [0.1.1] - 2024-02-28

### Added

- Support for defining `odra-build` dependency in `Cargo.toml` file of
  generated Odra projects.

## [0.1.0] - 2024-02-06

### Added

- `wasm-opt` to the build process. It is used to optimize the wasm file.
- Support for `.cargo/config.toml` target directory override.

### Fixed

- Inconsistent casing for `-c` argument
- Invalid fqn generation in some cases
- Schema generation for contracts

### Removed

- `cargo odra update` command. It is no longer needed, as it was the same
  as `cargo update`.

## [0.0.10] - 2023-12-13

### Fixed

- Fixed error that caused the template for odra 0.8.0 being downloaded for earlier
  versions of odra.

## [0.0.9] - 2023-07-19

### Added

- `generate` command validates if the module is already added to `lib.rs`.

## [0.0.8] - 2023-06-26

### Added

- Autocompletion.

### Changed

- Simplified wasm build process.
- Better dependencies management for the `builder`.

## [0.0.7] - 2023-05-22

### Added

- Support for Odra from crates.io.

## [0.0.6] - 2023-05-22

### Added

- `--contract-name` filter for `build` command.
- `--source` for `new` and `init` commands replacing `--branch`
- Support for workspaces.

## [0.0.5] - 2022-12-14

### Added

- `test` command can skip build using `--skip-build` flag.

## [0.0.4] - 2022-11-10

### Removed

- `backend` command is no logner needed.
- `gherkin` tests.

### Changed

- `generate` command doesn't override files anymore. It fails instead.
- `generate` uses hardcoded code instead of `flipper.rs` from Github.
- `Odra.toml` is now simpler. It doesn't have `name` anymore. `name` from
  `Cargo.toml` is used. A contract definition no longer needs `path` field.
  List of contract is no longer.
- Codebase is now lib based. It has `lib.rs` and the main bin is in `bin/cargo_odra.rs`.
- Error to use `thiserror`.
- Most of the file system functions is now in `commands.rs`.
- `just` replaced `make`.

### Added

- `paths.rs` for all paths releated operations.
- `template.rs` for code generation releated operations.
- `cli.rs` for `clap` code.
- `rustfmt.toml` with formatting rules.
- `init` and `new` can specify `git-branch` parameter.

## [0.0.3] - 2022-09-04

### Added

- new options - `--verbose` and `--quiet`, which will be passed to cargo commands.

### Changed

- Error handling, now each error has its own exit code.

## [0.0.2] - 2022-08-12

### Added

- `update` command.
- `backend list` command.
- Building a backend will now check if wasm target is installed.

### Changed

- `backend add` command now does not require `name` parameter - `package`
  will be used as name by default.
- `test` command now passes arguments supplied after `--` to cargo test
  without the need to type `-a`.

### Fixed

- `Unsupported dependency for backend` message that showed up on some
  configurations - thanks to **jrojek** from Odra.dev Discord for pointing
  this out.

## [0.0.1] - 2022-08-09

### Added

- `init` and `new` commands.
- `backend` command to manage backends with subcommands `add` and `remove`.
- `test` command with possibility to run tests against OdraVM and dedicated backend VM.
- `clean` command which removes temporary files generated by cargo-odra.
- `generate` command which can generate a new sample contract, ready to run and test.
- `CHANGELOG.md` and `README.md` files.
