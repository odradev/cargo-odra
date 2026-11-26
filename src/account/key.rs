//! Parsing of the keys the account commands take.

use std::{fmt, str::FromStr};

use casper_types::{account::AccountHash, AsymmetricType, PublicKey};

const ACCOUNT_HASH_PREFIX: &str = "account-hash-";

/// A key given on the command line: a public key, or an account hash.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyArg {
    /// A public key in hex: `01` + 64 characters (ed25519) or `02` + 66 characters (secp256k1).
    PublicKey(PublicKey),
    /// An account hash: 64 hex characters, with or without the `account-hash-` prefix.
    AccountHash(AccountHash),
}

impl KeyArg {
    /// The account hash the key stands for.
    pub fn account_hash(&self) -> AccountHash {
        match self {
            KeyArg::PublicKey(public_key) => public_key.to_account_hash(),
            KeyArg::AccountHash(account_hash) => *account_hash,
        }
    }
}

impl fmt::Display for KeyArg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KeyArg::PublicKey(public_key) => write!(f, "{}", public_key.to_hex_string()),
            KeyArg::AccountHash(account_hash) => {
                write!(f, "{}", account_hash.to_formatted_string())
            }
        }
    }
}

/// Why a key could not be parsed. The message says which forms are accepted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidKey {
    input: String,
    reason: &'static str,
}

impl fmt::Display for InvalidKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "'{}' is not a key: {}.\nGive a public key in hex (01 + 64 characters for ed25519, \
             02 + 66 characters for secp256k1), or an account hash (64 hex characters, with or \
             without the account-hash- prefix), e.g.\n  \
             016e7a1cdd29b0b78fd13af4c5598feff4ef2a97166e3ca6f2e4fbfccd80505bf1\n  \
             account-hash-0ee624f1bbaec23e6dc6b877815f322058045c45cdbf8ef53e7db2c58b0af309",
            self.input, self.reason
        )
    }
}

impl std::error::Error for InvalidKey {}

impl FromStr for KeyArg {
    type Err = InvalidKey;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let invalid = |reason| InvalidKey {
            input: input.to_string(),
            reason,
        };
        let key = input.trim();
        let (hex, prefixed) = match key.strip_prefix(ACCOUNT_HASH_PREFIX) {
            Some(hex) => (hex, true),
            None => (key, false),
        };
        if hex.is_empty() {
            return Err(invalid("it is empty"));
        }
        if !hex.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(invalid("it has characters that are not hex digits"));
        }
        match (hex.len(), prefixed) {
            (64, _) => {
                let mut bytes = [0u8; 32];
                for (i, byte) in bytes.iter_mut().enumerate() {
                    *byte = u8::from_str_radix(&hex[2 * i..2 * i + 2], 16)
                        .map_err(|_| invalid("it is not valid hex"))?;
                }
                Ok(KeyArg::AccountHash(AccountHash::new(bytes)))
            }
            (_, true) => Err(invalid("an account hash has 64 hex characters")),
            (66, false) if hex.starts_with("01") => PublicKey::from_hex(hex)
                .map(KeyArg::PublicKey)
                .map_err(|_| invalid("it is not a valid ed25519 public key")),
            (68, false) if hex.starts_with("02") => PublicKey::from_hex(hex)
                .map(KeyArg::PublicKey)
                .map_err(|_| invalid("it is not a valid secp256k1 public key")),
            (66, false) | (68, false) => Err(invalid(
                "a public key starts with 01 (ed25519, 66 characters) or 02 (secp256k1, 68 characters)",
            )),
            _ => Err(invalid(
                "it has neither the length of an account hash (64) nor of a public key (66 or 68)",
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use casper_types::SecretKey;

    use super::*;

    fn ed25519() -> PublicKey {
        PublicKey::from(&SecretKey::ed25519_from_bytes([7u8; 32]).unwrap())
    }

    fn secp256k1() -> PublicKey {
        PublicKey::from(&SecretKey::secp256k1_from_bytes([7u8; 32]).unwrap())
    }

    #[test]
    fn parses_public_keys() {
        for public_key in [ed25519(), secp256k1()] {
            let hex = public_key.to_hex_string();
            assert_eq!(
                hex.parse::<KeyArg>(),
                Ok(KeyArg::PublicKey(public_key.clone()))
            );
            assert_eq!(
                hex.parse::<KeyArg>().unwrap().account_hash(),
                public_key.to_account_hash()
            );
        }
        assert_eq!(ed25519().to_hex_string().len(), 66);
        assert_eq!(secp256k1().to_hex_string().len(), 68);
    }

    #[test]
    fn parses_account_hashes_with_and_without_the_prefix() {
        let account_hash = ed25519().to_account_hash();
        let formatted = account_hash.to_formatted_string();
        let bare = formatted.strip_prefix(ACCOUNT_HASH_PREFIX).unwrap();
        assert_eq!(bare.len(), 64);
        for input in [
            formatted.as_str(),
            bare,
            &bare.to_uppercase(),
            &format!(" {bare}\n"),
        ] {
            assert_eq!(
                input.parse::<KeyArg>(),
                Ok(KeyArg::AccountHash(account_hash)),
                "{input}"
            );
        }
    }

    #[test]
    fn the_examples_in_the_error_message_are_valid_keys() {
        let message = "x".parse::<KeyArg>().unwrap_err().to_string();
        let examples: Vec<&str> = message.lines().skip(2).map(str::trim).collect();
        assert_eq!(examples.len(), 2);
        let keys: Vec<KeyArg> = examples.iter().map(|e| e.parse().unwrap()).collect();
        assert!(matches!(keys[0], KeyArg::PublicKey(_)));
        assert_eq!(keys[0].account_hash(), keys[1].account_hash());
    }

    #[test]
    fn rejects_everything_else() {
        let bare = "a".repeat(64);
        let cases = [
            ("", "it is empty"),
            ("account-hash-", "it is empty"),
            (&"g".repeat(64), "it has characters that are not hex digits"),
            (
                &format!("account-hash-{}", "a".repeat(66)),
                "an account hash has 64 hex characters",
            ),
            (&format!("03{}", "a".repeat(64)), "a public key starts with"),
            (&format!("01{}", "a".repeat(66)), "a public key starts with"),
            (&bare[..63], "it has neither the length"),
            (
                &format!("02{}", "0".repeat(66)),
                "it is not a valid secp256k1 public key",
            ),
        ];
        for (input, reason) in cases {
            let error = input.parse::<KeyArg>().unwrap_err();
            assert!(
                error.reason.starts_with(reason),
                "{input}: {}",
                error.reason
            );
            assert!(error.to_string().contains("account-hash-"));
        }
    }
}
