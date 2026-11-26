//! Builds the transaction that runs the account manager wasm.
//!
//! casper-types has no public builder for a TransactionV1, so this assembles the payload the way
//! casper-client 5.0.0 does (`lib/cli/transaction_v1_builder.rs` and `fields_container.rs`): a
//! session with the wasm, `call` as the entry point, standard scheduling and the arguments, as
//! fields 0 to 3 of the payload.

use std::collections::{BTreeMap, BTreeSet};

use casper_types::{
    account::AccountHash,
    bytesrepr::{Bytes, ToBytes},
    Digest,
    InitiatorAddr,
    PricingMode,
    RuntimeArgs,
    SecretKey,
    TimeDiff,
    Timestamp,
    Transaction,
    TransactionArgs,
    TransactionEntryPoint,
    TransactionRuntimeParams,
    TransactionScheduling,
    TransactionTarget,
    TransactionV1,
    TransactionV1Payload,
};

use crate::errors::Error;

const ARGS_MAP_KEY: u16 = 0;
const TARGET_MAP_KEY: u16 = 1;
const ENTRY_POINT_MAP_KEY: u16 = 2;
const SCHEDULING_MAP_KEY: u16 = 3;

/// The parts of a session transaction that do not come from the wasm itself.
pub struct SessionParams {
    pub chain_name: String,
    pub timestamp: Timestamp,
    pub ttl: TimeDiff,
    /// The payment limit, in motes.
    pub payment_amount: u64,
    pub gas_price_tolerance: u8,
    /// The account the transaction runs as.
    pub initiator: InitiatorAddr,
}

impl SessionParams {
    /// The initiator for an account: its public key when the signer is the account's own key,
    /// otherwise its account hash (a multisig account the signer is an associated key of).
    pub fn initiator(account: AccountHash, signer: &SecretKey) -> InitiatorAddr {
        let public_key = casper_types::PublicKey::from(signer);
        if public_key.to_account_hash() == account {
            InitiatorAddr::PublicKey(public_key)
        } else {
            InitiatorAddr::AccountHash(account)
        }
    }
}

/// A session transaction running `wasm` with `args`, signed by `signer`. It installs nothing.
pub fn session_transaction(
    wasm: &[u8],
    args: RuntimeArgs,
    params: SessionParams,
    signer: &SecretKey,
) -> Result<Transaction, Error> {
    let target = TransactionTarget::Session {
        is_install_upgrade: false,
        module_bytes: Bytes::from(wasm.to_vec()),
        runtime: TransactionRuntimeParams::VmCasperV1,
    };
    let fields = fields(
        TransactionArgs::Named(args),
        target,
        TransactionEntryPoint::Call,
        TransactionScheduling::Standard,
    )?;
    let pricing_mode = PricingMode::PaymentLimited {
        payment_amount: params.payment_amount,
        gas_price_tolerance: params.gas_price_tolerance,
        standard_payment: true,
    };
    let payload = TransactionV1Payload::new(
        params.chain_name,
        params.timestamp,
        params.ttl,
        pricing_mode,
        params.initiator,
        fields,
    );
    let hash = Digest::hash(payload.to_bytes().map_err(serialization_error)?);
    let mut transaction = TransactionV1::new(hash.into(), payload, BTreeSet::new());
    transaction.sign(signer);
    Ok(Transaction::V1(transaction))
}

fn fields(
    args: TransactionArgs,
    target: TransactionTarget,
    entry_point: TransactionEntryPoint,
    scheduling: TransactionScheduling,
) -> Result<BTreeMap<u16, Bytes>, Error> {
    let mut map = BTreeMap::new();
    map.insert(
        ARGS_MAP_KEY,
        args.to_bytes().map_err(serialization_error)?.into(),
    );
    map.insert(
        TARGET_MAP_KEY,
        target.to_bytes().map_err(serialization_error)?.into(),
    );
    map.insert(
        ENTRY_POINT_MAP_KEY,
        entry_point.to_bytes().map_err(serialization_error)?.into(),
    );
    map.insert(
        SCHEDULING_MAP_KEY,
        scheduling.to_bytes().map_err(serialization_error)?.into(),
    );
    Ok(map)
}

fn serialization_error(error: casper_types::bytesrepr::Error) -> Error {
    Error::Transaction(format!("cannot serialize the transaction: {error}"))
}

/// Parses an amount of CSPR (`5`, `2.5`) into motes.
pub fn cspr_to_motes(cspr: &str) -> Result<u64, Error> {
    let invalid = || Error::Transaction(format!("'{cspr}' is not an amount of CSPR"));
    let (whole, fraction) = cspr.trim().split_once('.').unwrap_or((cspr.trim(), ""));
    if whole.is_empty() && fraction.is_empty() || fraction.len() > 9 {
        return Err(invalid());
    }
    if !whole
        .chars()
        .chain(fraction.chars())
        .all(|c| c.is_ascii_digit())
    {
        return Err(invalid());
    }
    let whole: u64 = if whole.is_empty() {
        0
    } else {
        whole.parse().map_err(|_| invalid())?
    };
    let fraction: u64 = format!("{fraction:0<9}").parse().map_err(|_| invalid())?;
    whole
        .checked_mul(1_000_000_000)
        .and_then(|motes| motes.checked_add(fraction))
        .filter(|motes| *motes > 0)
        .ok_or_else(invalid)
}

#[cfg(test)]
mod tests {
    use casper_types::{bytesrepr::FromBytes, PublicKey, TransactionHash};
    use odra_casper_account_manager::{AccountChange, ACCOUNT_MANAGER_WASM, SET_KEYS_ARG};

    use super::*;

    fn signer() -> SecretKey {
        SecretKey::ed25519_from_bytes([3u8; 32]).unwrap()
    }

    fn params(initiator: InitiatorAddr) -> SessionParams {
        SessionParams {
            chain_name: "casper-test".to_string(),
            timestamp: Timestamp::from(1_700_000_000_000),
            ttl: TimeDiff::from_seconds(300),
            payment_amount: 5_000_000_000,
            gas_price_tolerance: 1,
            initiator,
        }
    }

    #[test]
    fn builds_a_signed_session_with_the_wasm_and_the_args() {
        let signer = signer();
        let own = PublicKey::from(&signer);
        let other = AccountHash::new([9u8; 32]);
        let args = AccountChange::new()
            .set_key(other, 1)
            .deployment(1)
            .to_runtime_args();
        let initiator = SessionParams::initiator(own.to_account_hash(), &signer);
        assert_eq!(initiator, InitiatorAddr::PublicKey(own.clone()));
        let transaction = session_transaction(
            ACCOUNT_MANAGER_WASM,
            args.clone(),
            params(initiator),
            &signer,
        )
        .unwrap();
        let Transaction::V1(v1) = &transaction else {
            panic!("expected a TransactionV1");
        };
        v1.verify().expect("hash and signature are valid");
        assert_eq!(v1.approvals().len(), 1);
        assert_eq!(v1.approvals().iter().next().unwrap().signer(), &own);
        assert_eq!(v1.chain_name(), "casper-test");

        let payload = v1.payload();
        let target = TransactionTarget::from_bytes(payload.fields().get(&TARGET_MAP_KEY).unwrap())
            .unwrap()
            .0;
        match target {
            TransactionTarget::Session {
                is_install_upgrade,
                module_bytes,
                runtime,
            } => {
                assert!(!is_install_upgrade);
                assert_eq!(module_bytes.as_slice(), ACCOUNT_MANAGER_WASM);
                assert_eq!(runtime, TransactionRuntimeParams::VmCasperV1);
            }
            other => panic!("expected a session, got {other:?}"),
        }
        let sent_args = TransactionArgs::from_bytes(payload.fields().get(&ARGS_MAP_KEY).unwrap())
            .unwrap()
            .0;
        assert_eq!(sent_args, TransactionArgs::Named(args.clone()));
        assert!(args.get(SET_KEYS_ARG).is_some());
        let entry_point =
            TransactionEntryPoint::from_bytes(payload.fields().get(&ENTRY_POINT_MAP_KEY).unwrap())
                .unwrap()
                .0;
        assert_eq!(entry_point, TransactionEntryPoint::Call);

        // The node gets it as {"Version1": {...}}; it reads back to the same hash.
        let json = serde_json::to_value(&transaction).unwrap();
        assert!(json.get("Version1").is_some());
        let back: Transaction = serde_json::from_value(json).unwrap();
        assert_eq!(back.hash(), TransactionHash::V1(*v1.hash()));
    }

    #[test]
    fn a_multisig_account_is_initiated_by_its_account_hash() {
        let account = AccountHash::new([1u8; 32]);
        assert_eq!(
            SessionParams::initiator(account, &signer()),
            InitiatorAddr::AccountHash(account)
        );
    }

    #[test]
    fn parses_cspr() {
        assert_eq!(cspr_to_motes("5").unwrap(), 5_000_000_000);
        assert_eq!(cspr_to_motes("2.5").unwrap(), 2_500_000_000);
        assert_eq!(cspr_to_motes("0.000000001").unwrap(), 1);
        assert_eq!(cspr_to_motes(".5").unwrap(), 500_000_000);
        for invalid in [
            "",
            ".",
            "0",
            "-1",
            "1.0000000001",
            "1,5",
            "abc",
            "99999999999999999999",
        ] {
            assert!(cspr_to_motes(invalid).is_err(), "{invalid}");
        }
    }
}
