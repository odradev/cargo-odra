# cargo-odra

A cargo utility that helps to create, manage and test your smart contracts
written using Odra framework.   

## Table of Contents
* [Usage](#usage)
* [Commands](#backends)
* [Links](#links)
* [Contact](#contact)

## Prerequisites

- Rust toolchain installed (see [rustup.rs](https://rustup.rs/))
- wasm-strip tool installed (see [wabt](https://github.com/WebAssembly/wabt))
- wasm-opt tool installed (see [binaryen](https://github.com/WebAssembly/binaryen))

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
* `clean` - removes temporary files (builders and wasm files),
* `completions` - generates autocomplete script for given shell

To see exact syntax of each command, type `cargo odra command --help`.

## Workspaces

`cargo-odra` supports workspaces. To use it, simply move your `Odra.toml`
file into root of your workspace. If you have multiple odra crates in your
workspace, put all contracts in the same Odra.toml folder.

You can use a template to create a project with workspace:

```bash
$ cargo odra new --name myproject --template workspace && cd myproject
```

## Links

* [Odra framework repository](https://github.com/odradev/odra)
* [Odra docs](https://odra.dev/docs)

## Contact
Write **contact@odra.dev**

<div align="center">
by <a href="https://odra.dev">odra.dev<a>
</dev>
