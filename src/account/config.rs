//! Network and signer configuration of the account commands.
//!
//! It is read from the same variables Odra's livenet environment and odra-cli use, so a project
//! configured for odra-cli works as is: the process environment, the file named by
//! `ODRA_CASPER_LIVENET_ENV` (with the `.env` extension) and `.env` in the current directory.
//! Flags on the command line take precedence.

use std::path::PathBuf;

use crate::errors::Error;

pub const ENV_NODE_ADDRESS: &str = "ODRA_CASPER_LIVENET_NODE_ADDRESS";
pub const ENV_CHAIN_NAME: &str = "ODRA_CASPER_LIVENET_CHAIN_NAME";
pub const ENV_SECRET_KEY: &str = "ODRA_CASPER_LIVENET_SECRET_KEY_PATH";
pub const ENV_TTL: &str = "ODRA_CASPER_LIVENET_TTL";
pub const ENV_GAS_PRICE_TOLERANCE: &str = "ODRA_CASPER_LIVENET_GAS_PRICE_TOLERANCE";
pub const ENV_LIVENET_ENV_FILE: &str = "ODRA_CASPER_LIVENET_ENV";
pub const ENV_CSPR_CLOUD_AUTH_TOKEN: &str = "CSPR_CLOUD_AUTH_TOKEN";

/// Odra's default time to live of a transaction, in seconds.
const DEFAULT_TTL: u32 = 5 * 60;
const DEFAULT_GAS_PRICE_TOLERANCE: u8 = 1;

/// Values given as flags; each one overrides its variable.
#[derive(Debug, Default, Clone)]
pub struct Overrides {
    pub node_address: Option<String>,
    pub chain_name: Option<String>,
    pub secret_key: Option<PathBuf>,
}

/// Where to send requests and how to build transactions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetworkConfig {
    /// The node's address; `/rpc` is added unless it ends with it.
    pub node_address: String,
    /// Sent as the `Authorization` header when set (needed by `node.cspr.cloud`).
    pub auth_token: Option<String>,
}

impl NetworkConfig {
    /// The JSON-RPC endpoint.
    pub fn rpc_url(&self) -> String {
        let address = self.node_address.trim_end_matches('/');
        if address.ends_with("/rpc") {
            address.to_string()
        } else {
            format!("{address}/rpc")
        }
    }
}

/// What a command that sends a transaction needs on top of [`NetworkConfig`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SigningConfig {
    pub chain_name: String,
    pub secret_key: PathBuf,
    pub ttl_seconds: u32,
    pub gas_price_tolerance: u8,
}

type Lookup = Box<dyn Fn(&str) -> Option<String>>;

/// Reads the variables, after loading the env files (which never override a variable that is
/// already set).
pub struct Env {
    lookup: Lookup,
}

impl Env {
    /// The process environment, with the env files loaded first.
    pub fn load() -> Self {
        if let Ok(file) = std::env::var(ENV_LIVENET_ENV_FILE) {
            dotenvy::from_filename(PathBuf::from(file).with_extension("env")).ok();
        }
        dotenvy::dotenv().ok();
        Env {
            lookup: Box::new(|name| std::env::var(name).ok().filter(|value| !value.is_empty())),
        }
    }

    #[cfg(test)]
    pub fn from_pairs(pairs: &[(&str, &str)]) -> Self {
        let map: std::collections::HashMap<String, String> = pairs
            .iter()
            .map(|(name, value)| (name.to_string(), value.to_string()))
            .collect();
        Env {
            lookup: Box::new(move |name| map.get(name).cloned()),
        }
    }

    fn get(&self, name: &str) -> Option<String> {
        (self.lookup)(name)
    }

    fn required(&self, name: &str, flag: &str, value: Option<String>) -> Result<String, Error> {
        value
            .or_else(|| self.get(name))
            .ok_or_else(|| Error::MissingAccountConfig(format!("{flag} or {name}")))
    }

    /// The configuration for reading from the node.
    pub fn network(&self, overrides: &Overrides) -> Result<NetworkConfig, Error> {
        Ok(NetworkConfig {
            node_address: self.required(
                ENV_NODE_ADDRESS,
                "--node-address",
                overrides.node_address.clone(),
            )?,
            auth_token: self.get(ENV_CSPR_CLOUD_AUTH_TOKEN),
        })
    }

    /// The secret key path, if one is configured.
    pub fn secret_key(&self, overrides: &Overrides) -> Option<PathBuf> {
        overrides
            .secret_key
            .clone()
            .or_else(|| self.get(ENV_SECRET_KEY).map(PathBuf::from))
    }

    /// The configuration for sending a transaction.
    pub fn signing(&self, overrides: &Overrides) -> Result<SigningConfig, Error> {
        let chain_name =
            self.required(ENV_CHAIN_NAME, "--chain-name", overrides.chain_name.clone())?;
        let secret_key = self.secret_key(overrides).ok_or_else(|| {
            Error::MissingAccountConfig(format!("--secret-key or {ENV_SECRET_KEY}"))
        })?;
        let ttl_seconds = match self.get(ENV_TTL) {
            None => DEFAULT_TTL,
            Some(ttl) => ttl.parse().map_err(|_| {
                Error::MissingAccountConfig(format!(
                    "{ENV_TTL} as a number of seconds, not '{ttl}'"
                ))
            })?,
        };
        let gas_price_tolerance = match self.get(ENV_GAS_PRICE_TOLERANCE) {
            None => DEFAULT_GAS_PRICE_TOLERANCE,
            Some(tolerance) => tolerance.parse().map_err(|_| {
                Error::MissingAccountConfig(format!(
                    "{ENV_GAS_PRICE_TOLERANCE} as a number from 1 to 255, not '{tolerance}'"
                ))
            })?,
        };
        Ok(SigningConfig {
            chain_name,
            secret_key,
            ttl_seconds,
            gas_price_tolerance,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_odra_variables() {
        let env = Env::from_pairs(&[
            (ENV_NODE_ADDRESS, "http://localhost:11101"),
            (ENV_CHAIN_NAME, "casper-net-1"),
            (ENV_SECRET_KEY, "keys/secret_key.pem"),
            (ENV_TTL, "600"),
            (ENV_CSPR_CLOUD_AUTH_TOKEN, "token"),
        ]);
        let network = env.network(&Overrides::default()).unwrap();
        assert_eq!(network.rpc_url(), "http://localhost:11101/rpc");
        assert_eq!(network.auth_token.as_deref(), Some("token"));
        let signing = env.signing(&Overrides::default()).unwrap();
        assert_eq!(
            signing,
            SigningConfig {
                chain_name: "casper-net-1".to_string(),
                secret_key: PathBuf::from("keys/secret_key.pem"),
                ttl_seconds: 600,
                gas_price_tolerance: 1,
            }
        );
    }

    #[test]
    fn flags_override_variables() {
        let env = Env::from_pairs(&[
            (ENV_NODE_ADDRESS, "http://localhost:11101"),
            (ENV_CHAIN_NAME, "casper-net-1"),
        ]);
        let overrides = Overrides {
            node_address: Some("https://node.testnet.casper.network/rpc/".to_string()),
            chain_name: Some("casper-test".to_string()),
            secret_key: Some(PathBuf::from("other.pem")),
        };
        assert_eq!(
            env.network(&overrides).unwrap().rpc_url(),
            "https://node.testnet.casper.network/rpc"
        );
        let signing = env.signing(&overrides).unwrap();
        assert_eq!(signing.chain_name, "casper-test");
        assert_eq!(signing.secret_key, PathBuf::from("other.pem"));
        assert_eq!(signing.ttl_seconds, DEFAULT_TTL);
    }

    #[test]
    fn says_which_variable_is_missing() {
        let env = Env::from_pairs(&[]);
        let error = env.network(&Overrides::default()).unwrap_err();
        assert!(error.to_string().contains(ENV_NODE_ADDRESS));
        let error = Env::from_pairs(&[(ENV_CHAIN_NAME, "casper-test")])
            .signing(&Overrides::default())
            .unwrap_err();
        assert!(error.to_string().contains(ENV_SECRET_KEY));
        let error = Env::from_pairs(&[
            (ENV_CHAIN_NAME, "casper-test"),
            (ENV_SECRET_KEY, "k.pem"),
            (ENV_TTL, "5m"),
        ])
        .signing(&Overrides::default())
        .unwrap_err();
        assert!(error.to_string().contains("'5m'"));
    }
}
