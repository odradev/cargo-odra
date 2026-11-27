//! A small JSON-RPC client for the few Casper node methods the account commands use.
//!
//! The request and result shapes follow casper-client 5.0.0 (`lib/rpcs/v2_0_0`), which Odra
//! uses; the values themselves are casper-types' own serde forms.

use casper_types::{
    account::{Account, AccountHash},
    Transaction,
    TransactionHash,
    U512,
};
use serde::de::DeserializeOwned;
use serde_json::{json, Value};

use super::config::NetworkConfig;
use crate::errors::Error;

const GET_ACCOUNT_METHOD: &str = "state_get_account_info";
const QUERY_BALANCE_METHOD: &str = "query_balance";
const PUT_TRANSACTION_METHOD: &str = "account_put_transaction";
const GET_TRANSACTION_METHOD: &str = "info_get_transaction";

/// Sends JSON-RPC requests to a node.
pub struct RpcClient {
    config: NetworkConfig,
}

impl RpcClient {
    pub fn new(config: NetworkConfig) -> Self {
        RpcClient { config }
    }

    fn call(&self, method: &str, params: Value) -> Result<Value, Error> {
        let mut request = ureq::post(self.config.rpc_url())
            .config()
            .http_status_as_error(false)
            .build()
            .header("Content-Type", "application/json");
        if let Some(token) = &self.config.auth_token {
            request = request.header("Authorization", token);
        }
        let mut response = request
            .send_json(request_body(method, params))
            .map_err(|error| Error::Rpc(format!("{method}: {error}")))?;
        let status = response.status();
        let body: Value = response.body_mut().read_json().map_err(|error| {
            Error::Rpc(format!(
                "{method}: HTTP {status}, the response is not JSON: {error}"
            ))
        })?;
        result_of(method, body)
    }

    /// The account, as `state_get_account_info` returns it.
    pub fn account(&self, account_hash: AccountHash) -> Result<Account, Error> {
        let result = self.call(
            GET_ACCOUNT_METHOD,
            json!({
                "account_identifier": account_hash,
                "block_identifier": null,
            }),
        )?;
        field(GET_ACCOUNT_METHOD, &result, "account")
    }

    /// The balance of the account's main purse, in motes.
    pub fn balance(&self, account_hash: AccountHash) -> Result<U512, Error> {
        let result = self.call(
            QUERY_BALANCE_METHOD,
            json!({
                "state_identifier": null,
                "purse_identifier": { "main_purse_under_account_hash": account_hash },
            }),
        )?;
        field(QUERY_BALANCE_METHOD, &result, "balance")
    }

    /// Sends a signed transaction; returns its hash.
    pub fn put_transaction(&self, transaction: &Transaction) -> Result<TransactionHash, Error> {
        let result = self.call(
            PUT_TRANSACTION_METHOD,
            json!({ "transaction": transaction }),
        )?;
        field(PUT_TRANSACTION_METHOD, &result, "transaction_hash")
    }

    /// The outcome of a transaction, or `None` while it has not been executed.
    pub fn execution_result(&self, hash: TransactionHash) -> Result<Option<Execution>, Error> {
        let result = self.call(
            GET_TRANSACTION_METHOD,
            json!({ "transaction_hash": hash, "finalized_approvals": false }),
        )?;
        execution_result_of(&result)
    }
}

fn request_body(method: &str, params: Value) -> Value {
    json!({ "jsonrpc": "2.0", "id": 1, "method": method, "params": params })
}

/// The `result` of a JSON-RPC response, or its `error` as an [`Error::Rpc`].
fn result_of(method: &str, mut body: Value) -> Result<Value, Error> {
    if let Some(error) = body.get("error") {
        let message = error
            .get("message")
            .and_then(Value::as_str)
            .unwrap_or("unknown error");
        let data = error
            .get("data")
            .map(|data| match data {
                Value::String(data) => format!(": {data}"),
                other => format!(": {other}"),
            })
            .unwrap_or_default();
        let code = error
            .get("code")
            .map(|code| format!(" ({code})"))
            .unwrap_or_default();
        return Err(Error::Rpc(format!("{method}: {message}{code}{data}")));
    }
    body.get_mut("result").map(Value::take).ok_or_else(|| {
        Error::Rpc(format!(
            "{method}: the response has neither result nor error"
        ))
    })
}

fn field<T: DeserializeOwned>(method: &str, result: &Value, name: &str) -> Result<T, Error> {
    let value = result
        .get(name)
        .ok_or_else(|| Error::Rpc(format!("{method}: the result has no {name}")))?;
    serde_json::from_value(value.clone())
        .map_err(|error| Error::Rpc(format!("{method}: cannot read {name}: {error}")))
}

/// What the commands need of an execution result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Execution {
    /// Why the transaction failed, `None` if it succeeded.
    pub error_message: Option<String>,
    /// The gas it consumed, as the node reports it.
    pub consumed: Option<String>,
}

/// Reads the execution result from the JSON instead of casper-types' `ExecutionResult`: the
/// node adds a `parsed` field to the CLValues in the effects, which casper-types' serde does not
/// accept back.
fn execution_result_of(result: &Value) -> Result<Option<Execution>, Error> {
    let execution_result = match result
        .get("execution_info")
        .and_then(|info| info.get("execution_result"))
    {
        None | Some(Value::Null) => return Ok(None),
        Some(execution_result) => execution_result,
    };
    let text = |value: &Value| match value {
        Value::String(text) => Some(text.clone()),
        Value::Null => None,
        other => Some(other.to_string()),
    };
    if let Some(v2) = execution_result.get("Version2") {
        return Ok(Some(Execution {
            error_message: v2.get("error_message").and_then(text),
            consumed: v2.get("consumed").and_then(text),
        }));
    }
    if let Some(v1) = execution_result.get("Version1") {
        if let Some(failure) = v1.get("Failure") {
            return Ok(Some(Execution {
                error_message: Some(
                    failure
                        .get("error_message")
                        .and_then(text)
                        .unwrap_or_else(|| "failed".to_string()),
                ),
                consumed: failure.get("cost").and_then(text),
            }));
        }
        if let Some(success) = v1.get("Success") {
            return Ok(Some(Execution {
                error_message: None,
                consumed: success.get("cost").and_then(text),
            }));
        }
    }
    Err(Error::Rpc(format!(
        "{GET_TRANSACTION_METHOD}: unknown execution result: {execution_result}"
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    const ACCOUNT_HASH: &str =
        "account-hash-1f2e3d4c5b6a79881f2e3d4c5b6a79881f2e3d4c5b6a79881f2e3d4c5b6a7988";
    const OTHER_HASH: &str =
        "account-hash-a1b2c3d4e5f60718293a4b5c6d7e8f90a1b2c3d4e5f60718293a4b5c6d7e8f90";

    fn account_result() -> Value {
        json!({
            "api_version": "2.0.0",
            "account": {
                "account_hash": ACCOUNT_HASH,
                "named_keys": [],
                "main_purse": "uref-09480c3248ef76b603d386f3f4f8a5f87f597d4eaffd475433f861af187ab5db-007",
                "associated_keys": [
                    { "account_hash": ACCOUNT_HASH, "weight": 1 },
                    { "account_hash": OTHER_HASH, "weight": 2 }
                ],
                "action_thresholds": { "deployment": 1, "key_management": 2 }
            },
            "merkle_proof": "01000000"
        })
    }

    #[test]
    fn reads_an_account() {
        let account: Account = field(GET_ACCOUNT_METHOD, &account_result(), "account").unwrap();
        let other = AccountHash::from_formatted_str(OTHER_HASH).unwrap();
        assert_eq!(account.associated_keys().get(&other).unwrap().value(), 2);
        assert_eq!(account.action_thresholds().key_management().value(), 2);
    }

    #[test]
    fn request_params_match_casper_client() {
        let account_hash = AccountHash::from_formatted_str(ACCOUNT_HASH).unwrap();
        let body = request_body(
            GET_ACCOUNT_METHOD,
            json!({ "account_identifier": account_hash, "block_identifier": null }),
        );
        assert_eq!(body["method"], "state_get_account_info");
        assert_eq!(body["params"]["account_identifier"], ACCOUNT_HASH);
        let purse = json!({ "main_purse_under_account_hash": account_hash });
        assert_eq!(purse["main_purse_under_account_hash"], ACCOUNT_HASH);
    }

    #[test]
    fn reads_a_balance_and_a_transaction_hash() {
        let balance: U512 = field(
            QUERY_BALANCE_METHOD,
            &json!({ "api_version": "2.0.0", "balance": "1000000000000" }),
            "balance",
        )
        .unwrap();
        assert_eq!(balance, U512::from(1_000_000_000_000u64));
        let hash: TransactionHash = field(
            PUT_TRANSACTION_METHOD,
            &json!({ "transaction_hash": { "Version1": "aa".repeat(32) } }),
            "transaction_hash",
        )
        .unwrap();
        assert!(matches!(hash, TransactionHash::V1(_)));
    }

    #[test]
    fn reports_rpc_errors() {
        let body = json!({
            "jsonrpc": "2.0", "id": 1,
            "error": { "code": -32008, "message": "invalid transaction", "data": "account authorization invalid" }
        });
        let error = result_of(PUT_TRANSACTION_METHOD, body)
            .unwrap_err()
            .to_string();
        assert!(error.contains("account_put_transaction: invalid transaction (-32008): account authorization invalid"), "{error}");
        assert!(result_of("x", json!({})).is_err());
    }

    #[test]
    fn a_pending_transaction_has_no_result() {
        assert_eq!(
            execution_result_of(&json!({ "execution_info": null })).unwrap(),
            None
        );
        assert_eq!(
            execution_result_of(&json!({ "execution_info": { "block_hash": "00", "block_height": 1, "execution_result": null } })).unwrap(),
            None
        );
    }

    #[test]
    fn reads_a_failed_execution() {
        let result = json!({
            "execution_info": {
                "block_hash": "aa".repeat(32),
                "block_height": 10,
                "execution_result": {
                    "Version2": {
                        "initiator": { "AccountHash": ACCOUNT_HASH },
                        "error_message": "User error: 43",
                        "current_price": 1,
                        "limit": "5000000000",
                        "consumed": "100000000",
                        "cost": "5000000000",
                        "refund": "0",
                        "transfers": [],
                        "size_estimate": 100,
                        "effects": [{
                            "key": "balance-hold-01",
                            "kind": { "Write": { "CLValue": { "cl_type": "U512", "bytes": "00", "parsed": "0" } } }
                        }]
                    }
                }
            }
        });
        let execution = execution_result_of(&result).unwrap().unwrap();
        assert_eq!(execution.error_message.as_deref(), Some("User error: 43"));
        assert_eq!(execution.consumed.as_deref(), Some("100000000"));
    }

    #[test]
    fn reads_a_successful_execution() {
        let result = json!({ "execution_info": { "execution_result": {
            "Version2": { "error_message": null, "consumed": "123" }
        } } });
        assert_eq!(
            execution_result_of(&result).unwrap(),
            Some(Execution {
                error_message: None,
                consumed: Some("123".to_string())
            })
        );
        assert!(execution_result_of(
            &json!({ "execution_info": { "execution_result": { "Version9": {} } } })
        )
        .is_err());
    }
}
