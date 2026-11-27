//! `cargo odra account`: reads an account's associated keys and thresholds, and changes them.

use std::{
    collections::BTreeSet,
    io::{self, IsTerminal},
    path::Path,
    thread,
    time::{Duration, Instant},
};

use casper_types::{
    account::AccountHash,
    PublicKey,
    SecretKey,
    TimeDiff,
    Timestamp,
    TransactionHash,
};
use odra_casper_account_manager::{plan, AccountChange, ACCOUNT_MANAGER_WASM};

use crate::{
    account::{
        account_state,
        config::{Env, Overrides},
        key::KeyArg,
        render,
        rpc::RpcClient,
        transaction::{cspr_to_motes, session_transaction, SessionParams},
    },
    errors::Error,
    log,
};

/// How often to ask the node whether the transaction was executed.
const POLL_INTERVAL: Duration = Duration::from_secs(3);

fn load_secret_key(path: &Path) -> Result<SecretKey, Error> {
    SecretKey::from_file(path)
        .map_err(|error| Error::AccountSecretKey(format!("{}: {error}", path.display())))
}

/// The full hash, as explorers show it (the `Display` of a hash abbreviates it; the `Debug` of
/// a `Digest` is its hex).
fn transaction_hash_hex(hash: &TransactionHash) -> String {
    match hash {
        TransactionHash::V1(hash) => format!("{:?}", hash.inner()),
        TransactionHash::Deploy(hash) => format!("{:?}", hash.inner()),
    }
}

/// `cargo odra account info`: prints the keys, weights, thresholds and balance of `key`'s
/// account, or of the configured secret key's account.
pub fn info(overrides: &Overrides, key: Option<KeyArg>) -> Result<(), Error> {
    let env = Env::load();
    let client = RpcClient::new(env.network(overrides)?);
    let own_key = match env.secret_key(overrides) {
        Some(path) => Some(PublicKey::from(&load_secret_key(&path)?).to_account_hash()),
        None => None,
    };
    let account_hash = match (&key, own_key) {
        (Some(key), _) => key.account_hash(),
        (None, Some(own)) => own,
        (None, None) => {
            return Err(Error::MissingAccountConfig(
                "an account to show, or --secret-key or ODRA_CASPER_LIVENET_SECRET_KEY_PATH"
                    .to_string(),
            ))
        }
    };
    let account = client.account(account_hash)?;
    let balance = client.balance(account_hash)?;
    let own_keys: BTreeSet<_> = own_key.into_iter().collect();
    println!("Account {}", account_hash.to_formatted_string());
    print!(
        "{}",
        render::state(&account_state(&account), &own_keys, Some(balance))
    );
    Ok(())
}

/// What a command that changes an account needs besides the change itself.
pub struct ChangeOptions {
    pub overrides: Overrides,
    /// The account to change; the secret key's own account when not given.
    pub account: Option<KeyArg>,
    /// The payment limit, in CSPR.
    pub gas: String,
    /// Answer yes to the warnings.
    pub yes: bool,
    /// Wait at most this long for the execution result.
    pub timeout: Duration,
}

/// Checks the change against the account, shows it, asks about the warnings, sends the
/// account manager wasm and waits for the result.
pub fn change(change: AccountChange, options: ChangeOptions) -> Result<(), Error> {
    let env = Env::load();
    let client = RpcClient::new(env.network(&options.overrides)?);
    let signing = env.signing(&options.overrides)?;
    let payment_amount = cspr_to_motes(&options.gas)?;
    let secret_key = load_secret_key(&signing.secret_key)?;
    let signer: AccountHash = PublicKey::from(&secret_key).to_account_hash();
    let account_hash = options
        .account
        .as_ref()
        .map(KeyArg::account_hash)
        .unwrap_or(signer);

    let state = account_state(&client.account(account_hash)?);
    let own_keys: BTreeSet<_> = [signer].into_iter().collect();
    let plan = plan(&state, &change, &own_keys)
        .map_err(|error| Error::InvalidAccountChange(render::validation_error(&error)))?;

    println!("Account {}", account_hash.to_formatted_string());
    print!("{}", render::diff(&plan.before, &plan.after, &own_keys));
    for warning in &plan.warnings {
        log::warn(render::warning(warning));
    }
    render::confirm(
        &plan.warnings,
        options.yes,
        io::stdin().is_terminal(),
        &mut io::stdin().lock(),
        &mut io::stdout(),
    )?;

    let transaction = session_transaction(
        ACCOUNT_MANAGER_WASM,
        change.to_runtime_args(),
        SessionParams {
            chain_name: signing.chain_name,
            timestamp: Timestamp::now(),
            ttl: TimeDiff::from_seconds(signing.ttl_seconds),
            payment_amount,
            gas_price_tolerance: signing.gas_price_tolerance,
            initiator: SessionParams::initiator(account_hash, &secret_key),
        },
        &secret_key,
    )?;
    let hash = client.put_transaction(&transaction)?;
    let hash_hex = transaction_hash_hex(&hash);
    log::info(format!(
        "Sent transaction {hash_hex}, waiting for its execution..."
    ));

    let started = Instant::now();
    let result = loop {
        if let Some(result) = client.execution_result(hash)? {
            break result;
        }
        if started.elapsed() > options.timeout {
            return Err(Error::AccountTransactionTimeout(format!(
                "{hash_hex} was not executed within {} s; check it later on the node",
                options.timeout.as_secs()
            )));
        }
        thread::sleep(POLL_INTERVAL);
    };
    let consumed = result
        .consumed
        .map(|gas| format!(" (gas consumed: {gas})"))
        .unwrap_or_default();
    if let Some(message) = result.error_message {
        return Err(Error::AccountTransactionFailed(format!(
            "{hash_hex}{consumed}: {}",
            render::execution_error(&message)
        )));
    }
    log::info(format!("Transaction {hash_hex} executed{consumed}."));

    let after = account_state(&client.account(account_hash)?);
    print!("{}", render::state(&after, &own_keys, None));
    if after != plan.after {
        log::warn("the account differs from the planned configuration; someone else may have changed it meanwhile");
    }
    Ok(())
}
