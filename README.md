# cargo-odra

A cargo utility that helps to create, manage and test your smart contracts
written using Odra framework.   

## Table of Contents
* [Usage](#usage)
* [Commands](#backends)
* [Links](#links)
* [Contact](#contact)

## Usage

To create a new project use `init` or `new` command:

```bash
$ cargo odra new --name myproject && cd myproject
```

A sample contract - Flipper - will be created for you, with some sample tests.
To run them against MockVM, simply type:

```
$ cargo odra test
```

If you want to test your code using real backend VM type:

```
$ cargo odra test -b casper
```

## Commands

* `new` - creates a new project in a new folder,
* `init` - creates a new project in an existing, empty folder,
* `build` - builds the contracts, generates wasm files,
* `test` - runs tests,
* `generate` - generates sample contract,
* `clean` - removes temporary files (builders and wasm files),
* `update` - runs cargo update on project and backends.

To see exact syntax of each command, type `cargo odra command --help`.

## Links

* [Odra](https://github.com/odradev/odra)
* [Cargo Odra](https://github.com/odradev/cargo-odra)
* [Odra Template](https://github.com/odradev/odra-template)
* [Example Contract: Owned Token](https://github.com/odradev/owned-token)
* [Odra Casper](https://github.com/odradev/odra-casper)
* [Original Proposal for Odra Framework](https://github.com/odradev/odra-proposal)

## Contact
Write **contact@odra.dev**

---
<div align="center">
by <a href="https://odra.dev">odra.dev<a>
</dev>
