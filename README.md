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
