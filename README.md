# cargo-odra

A cargo utility that helps to create, manage and test your smart contracts
written using Odra framework.

This is the tooling only. The framework you write contracts against is
[Odra](https://github.com/odradev/odra), and the guides are at
[odra.dev/docs](https://odra.dev/docs) — start there if you are new.

## Table of Contents

* [Prerequisites](#prerequisites)
* [Install](#install)
* [Usage](#usage)
* [Commands](#commands)
* [Workspaces](#workspaces)
* [Contracts from dependency crates](#contracts-from-dependency-crates)
* [Templates](#templates)
* [Working with an AI agent](#working-with-an-ai-agent)
* [Links](#links)
* [Contact](#contact)

## Prerequisites

- Rust toolchain installed (see [rustup.rs](https://rustup.rs/))
- The `wasm32-unknown-unknown` target: `rustup target add wasm32-unknown-unknown`
- wasm-strip tool installed (see [wabt](https://github.com/WebAssembly/wabt))
- wasm-opt tool installed (see [binaryen](https://github.com/WebAssembly/binaryen))

The [Installation guide](https://odra.dev/docs/getting-started/installation) walks through all of
them and is kept up to date with the current release. On Ubuntu or WSL, use
[Ubuntu / WSL setup](https://odra.dev/docs/getting-started/ubuntu-wsl-setup) — it has the exact
commands, verified on a clean machine.

> [!IMPORTANT]
> `wasm-opt` must be **binaryen 121 or newer**. `cargo-odra` passes `--llvm-memory-copy-fill-lowering`,
> which older versions reject — and the resulting error says `is it installed?` rather than pointing
> at the version. Ubuntu 24.04 ships 108 and 26.04 ships 120, so install it from the
> [binaryen releases](https://github.com/WebAssembly/binaryen/releases) rather than from your
> distribution's package manager.

## Install

Use `cargo` to install `cargo-odra`:

```bash
$ cargo install cargo-odra --locked
```

## Usage

To create a new project use `init` or `new` command:

```bash
$ cargo odra new --name myproject && cd myproject
```

A sample contract - Flipper - will be created for you, with some sample tests.
To run them against OdraVM, simply type:

```bash
$ cargo odra test
```

If you want to test your code using real backend VM type:

```bash
$ cargo odra test -b casper
```

## Commands

* `new` - creates a new project in a new folder,
* `init` - creates a new project in an existing, empty folder,
* `build` - builds the contracts, generates wasm files,
* `test` - runs tests,
* `generate` - generates sample contract,
* `list-templates` - lists available templates,
* `clean` - removes temporary files (builders and wasm files),
* `completions` - generates autocomplete script for given shell

To see exact syntax of each command, type `cargo odra command_name --help`.

## Workspaces

`cargo-odra` supports workspaces. To use it, simply move your `Odra.toml`
file into root of your workspace. If you have multiple odra crates in your
workspace, put all contracts in the same Odra.toml folder.

You can use a template to create a project with workspace:

```bash
$ cargo odra new --name myproject --template workspace && cd myproject
```

## Contracts from dependency crates

A contract does not have to live in your project. If the crate that defines it is a dependency,
put its crate name as the first segment of the `fqn`:

```toml
[[contracts]]
fqn = "odra_modules::erc20::Erc20"
```

```toml
# Cargo.toml of the crate that builds it
[dependencies]
odra-modules = "3.0.0"
```

`cargo odra build -c Erc20` then builds `wasm/Erc20.wasm` and `cargo odra schema -c Erc20`
writes `resources/casper_contract_schemas/erc20_schema.json`, exactly as for a local contract.
The build is driven by the crate that depends on the contract's crate: the project crate in a
single crate project, or the first workspace member that lists the dependency.

That crate has to reference the dependency somewhere in its Rust code — rust links a dependency
only where it is used, so a crate that never mentions `odra_modules` would produce a wasm without
a single entry point. If it does not use the crate already, add a plain import to its build
binaries:

```rust
// bin/build_contract.rs and bin/build_schema.rs
use odra_modules;
```

`cargo odra build` stops with an explanation instead of building an empty wasm when the reference
is missing.

Requires Odra 3.0.0 or newer. Odra gates a module's wasm entry points on
`ODRA_MODULE`; since 3.0.0 the value may be crate-qualified (`odra_modules::Erc20`), so two
crates defining a module of the same name no longer clash. `cargo-odra` passes the qualified
value only when the project's Odra is 3.0.0 or newer (or a git/path dependency); older releases
keep getting the bare struct name.

Two contracts with the same struct name still cannot be listed together — the wasm and the
schema files are named after the struct.

## Templates

Templates are not bundled with this crate. `new`, `init` and `generate` download them at runtime
from the framework repository — [`templates/`](https://github.com/odradev/odra/tree/HEAD/templates)
in [odradev/odra](https://github.com/odradev/odra), indexed by `templates/templates.json`.

That means a template fix lands in the `odra` repo, not here, and `cargo odra list-templates` is
always the accurate list for the version you are targeting.

## Working with an AI agent

If you develop with Claude Code, install the
[Odra plugin](https://github.com/odradev/odradev-plugins). It drives this tool the way the
documentation describes — scaffolding, building, testing, and deploying — instead of guessing at
flags:

```
/plugin marketplace add odradev/odradev-plugins
/plugin install odra-plugin@odradev-plugins
```

Agents without the plugin should start from [odra.dev/llms.txt](https://odra.dev/llms.txt), an index
of the whole documentation set.

## Links

* [Odra framework repository](https://github.com/odradev/odra) — the framework, project templates
  and examples
* [Odra docs](https://odra.dev/docs) — guides and tutorials; see
  [Cargo Odra](https://odra.dev/docs/basics/cargo-odra) for these commands in depth
* [Odra API reference](https://docs.rs/odra/latest/odra/)
* [Odra Claude Code plugin](https://github.com/odradev/odradev-plugins) — skills for agentic Odra
  development
* [Discord](https://discord.com/invite/Mm5ABc9P8k)

## Contact

Write **contact@odra.dev**

<div align="center">
by <a href="https://odra.dev">odra.dev<a>
</dev>
