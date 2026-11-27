//! What the account commands print: the configuration before and after a change, the warnings
//! and the errors, and the confirmation prompt.

use std::{
    collections::BTreeSet,
    fmt::Write as _,
    io::{BufRead, Write},
};

use casper_types::{account::AccountHash, U512};
use odra_casper_account_manager::{
    AccountState,
    Error as WasmError,
    ThresholdKind,
    ValidationError,
    Warning,
};

use crate::errors::Error;

fn hash(key: &AccountHash) -> String {
    key.to_formatted_string()
}

fn threshold_name(kind: ThresholdKind) -> &'static str {
    match kind {
        ThresholdKind::Deployment => "deployment",
        ThresholdKind::KeyManagement => "key_management",
    }
}

/// The configuration of an account, as `account info` prints it.
pub fn state(
    state: &AccountState,
    own_keys: &BTreeSet<AccountHash>,
    balance: Option<U512>,
) -> String {
    let mut out = String::new();
    writeln!(
        out,
        "Associated keys (total weight {}):",
        state.total_weight()
    )
    .unwrap();
    for (key, weight) in &state.associated_keys {
        let own = if own_keys.contains(key) {
            "  (your key)"
        } else {
            ""
        };
        writeln!(out, "  {}  weight {weight}{own}", hash(key)).unwrap();
    }
    writeln!(
        out,
        "Thresholds: deployment {}, key_management {}",
        state.deployment, state.key_management
    )
    .unwrap();
    if let Some(balance) = balance {
        writeln!(out, "Balance: {} motes", balance).unwrap();
    }
    out
}

/// The configuration before and after a change, side by side.
pub fn diff(
    before: &AccountState,
    after: &AccountState,
    own_keys: &BTreeSet<AccountHash>,
) -> String {
    let mut out = String::new();
    writeln!(out, "{:<79} {:>6} {:>6}", "key", "before", "after").unwrap();
    let keys: BTreeSet<_> = before
        .associated_keys
        .keys()
        .chain(after.associated_keys.keys())
        .collect();
    for key in keys {
        let weight = |state: &AccountState| {
            state
                .associated_keys
                .get(key)
                .map(u8::to_string)
                .unwrap_or_else(|| "-".to_string())
        };
        let (old, new) = (weight(before), weight(after));
        let note = match (
            before.associated_keys.get(key),
            after.associated_keys.get(key),
        ) {
            (None, Some(_)) => "  added",
            (Some(_), None) => "  removed",
            (Some(a), Some(b)) if a != b => "  changed",
            _ => "",
        };
        let own = if own_keys.contains(key) {
            " (your key)"
        } else {
            ""
        };
        writeln!(out, "{:<79} {old:>6} {new:>6}{note}{own}", hash(key)).unwrap();
    }
    for (name, old, new) in [
        ("deployment threshold", before.deployment, after.deployment),
        (
            "key_management threshold",
            before.key_management,
            after.key_management,
        ),
        ("total weight", before.total_weight(), after.total_weight()),
    ] {
        let note = if old != new { "  changed" } else { "" };
        writeln!(out, "{name:<79} {old:>6} {new:>6}{note}").unwrap();
    }
    out
}

/// Why the node would reject the change.
pub fn validation_error(error: &ValidationError) -> String {
    match error {
        ValidationError::EmptyChange => "the command asks for no change".to_string(),
        ValidationError::ZeroWeight(key) => {
            format!("{} would get weight 0; remove the key instead", hash(key))
        }
        ValidationError::ZeroThreshold(kind) => {
            format!("the {} threshold cannot be 0", threshold_name(*kind))
        }
        ValidationError::SetAndRemoved(key) => {
            format!("{} is both set and removed", hash(key))
        }
        ValidationError::RemoveMissingKey(key) => {
            format!("{} is not an associated key of the account", hash(key))
        }
        ValidationError::TooManyKeys { count } => format!(
            "the account would have {count} associated keys, more than the {} allowed",
            odra_casper_account_manager::MAX_ASSOCIATED_KEYS
        ),
        ValidationError::ThresholdAboveTotalWeight {
            kind,
            threshold,
            total_weight,
        } => format!(
            "the {} threshold ({threshold}) would be above the total weight of all keys \
             ({total_weight}): nobody could reach it, and the node rejects it",
            threshold_name(*kind)
        ),
        ValidationError::DeploymentAboveKeyManagement {
            deployment,
            key_management,
        } => format!(
            "the deployment threshold ({deployment}) would be above the key_management threshold \
             ({key_management}), which the node rejects"
        ),
    }
}

/// What the person sending the change should know before confirming it.
pub fn warning(warning: &Warning) -> String {
    match warning {
        Warning::OwnKeyRemoved(key) => format!(
            "you are removing your own key {}: it will no longer be able to sign transactions \
             for this account",
            hash(key)
        ),
        Warning::OwnKeysBelowThreshold {
            kind: ThresholdKind::Deployment,
            weight,
            threshold,
        } => format!(
            "after the change your key(s) weigh {weight}, below the deployment threshold \
             ({threshold}): you will no longer be able to sign transactions for this account on \
             your own"
        ),
        Warning::OwnKeysBelowThreshold {
            kind: ThresholdKind::KeyManagement,
            weight,
            threshold,
        } => format!(
            "after the change your key(s) weigh {weight}, below the key_management threshold \
             ({threshold}): you will no longer be able to change the account's keys or \
             thresholds on your own"
        ),
        Warning::NoSpareKeyManagementWeight { weight } => format!(
            "the total weight of all keys ({weight}) equals the key_management threshold: losing \
             any key with weight locks the account's key management for good"
        ),
        Warning::NeedsMoreSignatures { weight, required } => format!(
            "your key(s) weigh {weight}, below the current key_management threshold ({required}): \
             the transaction will fail unless more associated keys sign it"
        ),
    }
}

/// What a reverted account manager transaction means, from its error message
/// (`User error: <code>`).
pub fn execution_error(message: &str) -> String {
    let code = message.rsplit_once("User error: ").and_then(|(_, rest)| {
        let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
        digits.parse::<u16>().ok()
    });
    match code.and_then(WasmError::from_code) {
        Some(error) => format!("{message}: {}", error.description()),
        None => message.to_string(),
    }
}

/// Asks to go on after warnings. `--yes` answers for the person; without a terminal and without
/// `--yes` the change is refused.
pub fn confirm(
    warnings: &[Warning],
    yes: bool,
    interactive: bool,
    input: &mut impl BufRead,
    output: &mut impl Write,
) -> Result<(), Error> {
    if warnings.is_empty() || yes {
        return Ok(());
    }
    if !interactive {
        return Err(Error::AccountChangeNotConfirmed(
            "there are warnings and no terminal to confirm them; run with --yes to send anyway"
                .to_string(),
        ));
    }
    write!(output, "Send the transaction anyway? [y/N] ").map_err(Error::IO)?;
    output.flush().map_err(Error::IO)?;
    let mut answer = String::new();
    input.read_line(&mut answer).map_err(Error::IO)?;
    match answer.trim().to_lowercase().as_str() {
        "y" | "yes" => Ok(()),
        _ => Err(Error::AccountChangeNotConfirmed(
            "not confirmed".to_string(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use odra_casper_account_manager::{plan, AccountChange};

    use super::*;

    fn key(n: u8) -> AccountHash {
        AccountHash::new([n; 32])
    }

    fn own() -> BTreeSet<AccountHash> {
        [key(1)].into_iter().collect()
    }

    #[test]
    fn shows_the_change_side_by_side() {
        let change = AccountChange::new()
            .set_key(key(2), 2)
            .deployment(2)
            .key_management(3);
        let plan = plan(&AccountState::new(key(1)), &change, &own()).unwrap();
        let out = diff(&plan.before, &plan.after, &own());
        let lines: Vec<&str> = out.lines().collect();
        assert!(lines[0].starts_with("key"));
        assert!(
            lines[1].starts_with(&hash(&key(1))) && lines[1].ends_with("1      1 (your key)"),
            "{}",
            lines[1]
        );
        assert!(
            lines[2].starts_with(&hash(&key(2))) && lines[2].ends_with("-      2  added"),
            "{}",
            lines[2]
        );
        assert!(
            lines[3].starts_with("deployment threshold") && lines[3].ends_with("1      2  changed")
        );
        assert!(lines[4].ends_with("1      3  changed"));
        assert!(lines[5].starts_with("total weight") && lines[5].ends_with("1      3  changed"));
    }

    #[test]
    fn shows_a_removed_key() {
        let mut state = AccountState::new(key(1));
        state.associated_keys.insert(key(2), 1);
        let change = AccountChange::new().remove_key(key(1));
        let plan = plan(&state, &change, &own()).unwrap();
        let out = diff(&plan.before, &plan.after, &own());
        assert!(
            out.lines()
                .nth(1)
                .unwrap()
                .ends_with("1      -  removed (your key)"),
            "{out}"
        );
        assert!(plan
            .warnings
            .iter()
            .any(|w| warning(w).contains("removing your own key")));
    }

    #[test]
    fn info_lists_keys_and_thresholds() {
        let out = state(&AccountState::new(key(1)), &own(), Some(U512::from(42u8)));
        assert!(out.contains("weight 1  (your key)"));
        assert!(out.contains("Thresholds: deployment 1, key_management 1"));
        assert!(out.contains("Balance: 42 motes"));
    }

    #[test]
    fn every_warning_and_error_has_a_message() {
        let warnings = [
            Warning::OwnKeyRemoved(key(1)),
            Warning::OwnKeysBelowThreshold {
                kind: ThresholdKind::Deployment,
                weight: 1,
                threshold: 2,
            },
            Warning::OwnKeysBelowThreshold {
                kind: ThresholdKind::KeyManagement,
                weight: 1,
                threshold: 2,
            },
            Warning::NoSpareKeyManagementWeight { weight: 2 },
            Warning::NeedsMoreSignatures {
                weight: 1,
                required: 2,
            },
        ];
        for w in &warnings {
            assert!(!warning(w).is_empty());
        }
        assert!(warning(&warnings[1]).contains("sign transactions"));
        let errors = [
            ValidationError::EmptyChange,
            ValidationError::ZeroWeight(key(1)),
            ValidationError::ZeroThreshold(ThresholdKind::Deployment),
            ValidationError::SetAndRemoved(key(1)),
            ValidationError::RemoveMissingKey(key(1)),
            ValidationError::TooManyKeys { count: 101 },
            ValidationError::ThresholdAboveTotalWeight {
                kind: ThresholdKind::KeyManagement,
                threshold: 4,
                total_weight: 3,
            },
            ValidationError::DeploymentAboveKeyManagement {
                deployment: 3,
                key_management: 2,
            },
        ];
        for e in &errors {
            assert!(!validation_error(e).is_empty());
        }
        assert!(validation_error(&errors[6])
            .contains("(4) would be above the total weight of all keys (3)"));
        assert!(validation_error(&errors[7]).contains("deployment threshold (3)"));
    }

    #[test]
    fn explains_a_reverted_transaction() {
        assert_eq!(
            execution_error("User error: 43"),
            "User error: 43: a threshold would be above the total weight of all keys"
        );
        assert_eq!(execution_error("Out of gas error"), "Out of gas error");
        assert_eq!(execution_error("User error: 7000"), "User error: 7000");
    }

    #[test]
    fn asks_before_sending_with_warnings() {
        let warnings = [Warning::OwnKeyRemoved(key(1))];
        let mut out = Vec::new();
        assert!(confirm(&[], false, false, &mut "".as_bytes(), &mut out).is_ok());
        assert!(confirm(&warnings, true, false, &mut "".as_bytes(), &mut out).is_ok());
        let refused =
            confirm(&warnings, false, false, &mut "y\n".as_bytes(), &mut out).unwrap_err();
        assert!(refused.to_string().contains("--yes"));
        assert!(confirm(&warnings, false, true, &mut "y\n".as_bytes(), &mut out).is_ok());
        assert!(confirm(&warnings, false, true, &mut "Yes\n".as_bytes(), &mut out).is_ok());
        assert!(confirm(&warnings, false, true, &mut "\n".as_bytes(), &mut out).is_err());
        assert!(confirm(&warnings, false, true, &mut "n\n".as_bytes(), &mut out).is_err());
        assert!(String::from_utf8(out).unwrap().contains("[y/N]"));
    }
}
