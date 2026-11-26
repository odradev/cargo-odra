//! Casper account management: reading an account's associated keys and action thresholds, and
//! changing them with the account manager wasm of `odra-casper-account-manager`.

pub mod config;
pub mod key;
pub mod render;
pub mod rpc;
pub mod transaction;

use casper_types::account::Account;
use odra_casper_account_manager::AccountState;

/// The multisig configuration of an account read from the node.
pub fn account_state(account: &Account) -> AccountState {
    AccountState {
        associated_keys: account
            .associated_keys()
            .iter()
            .map(|(key, weight)| (*key, weight.value()))
            .collect(),
        deployment: account.action_thresholds().deployment().value(),
        key_management: account.action_thresholds().key_management().value(),
    }
}
