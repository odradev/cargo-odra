//! Cargo Odra is a tool that helps you creating, maintaining, testing and building smart contracts
//! developed using the Odra framework.
//!
//! To see examples on how to use cargo odra, visit project's
//! [Github Page](https://github.com/odradev/cargo-odra).

mod actions;
mod cargo_toml;
pub mod cli;
mod command;
mod consts;
mod errors;
mod log;
mod odra_toml;
mod paths;
mod template;

// TODO: Name things correctly.
// TODO: Use casing from odra, so erc20 is not erc_20.
