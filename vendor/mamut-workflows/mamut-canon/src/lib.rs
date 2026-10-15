// SPDX-License-Identifier: Apache-2.0

//! JCS canonicalization and SHA-256 fingerprints.
//!
//! [`fingerprint`] is the hash the later executor will put on an execution
//! manifest. It hashes [RFC 8785] bytes: UTF-8, object keys ordered by UTF-16
//! code units, no whitespace, and ECMAScript number serialization. The same
//! JSON produces the same bytes on native and `wasm32-unknown-unknown`. There
//! is no clock and no random source.
//!
//! [RFC 8785]: https://www.rfc-editor.org/rfc/rfc8785

#![deny(missing_docs)]
#![forbid(unsafe_code)]

use std::fmt;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Number, Value};
use sha2::{Digest, Sha256};

/// Hash algorithm recorded on a [`Fingerprint`].
pub const ALGORITHM: &str = "sha256";

/// Canonicalization recorded on a [`Fingerprint`].
pub const CANONICALIZATION: &str = "rfc8785-jcs";

/// `fingerprint/1` record: SHA-256 over JCS bytes, lowercase hex.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Fingerprint {
    /// Always [`ALGORITHM`].
    pub algorithm: String,
    /// Always [`CANONICALIZATION`].
    pub canonicalization: String,
    /// 64 lowercase hex characters.
    pub value: String,
}

/// JSON could not be canonicalized.
#[derive(Debug, PartialEq, Eq)]
pub enum CanonError {
    /// The text is not a JSON value.
    Json(String),
    /// A number is not a finite IEEE 754 binary64 value.
    NonFiniteNumber,
}

impl fmt::Display for CanonError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Json(message) => write!(f, "json: {message}"),
            Self::NonFiniteNumber => f.write_str("number is not a finite IEEE 754 value"),
        }
    }
}

impl std::error::Error for CanonError {}

/// Canonicalize `json` to RFC 8785 bytes.
pub fn canonicalize(json: &str) -> Result<Vec<u8>, CanonError> {
    let value = serde_json::from_str(json).map_err(|error| CanonError::Json(error.to_string()))?;
    canonicalize_value(&value)
}

/// Canonicalize a JSON value to RFC 8785 bytes.
pub fn canonicalize_value(value: &Value) -> Result<Vec<u8>, CanonError> {
    let normalized = normalize(value)?;
    json_canon::to_vec(&normalized).map_err(|error| CanonError::Json(error.to_string()))
}

/// SHA-256 of `bytes`, as 64 lowercase hex characters.
pub fn sha256_hex(bytes: &[u8]) -> String {
    hex_encode(&Sha256::digest(bytes))
}

/// Fingerprint of a JSON document.
pub fn fingerprint(json: &str) -> Result<Fingerprint, CanonError> {
    Ok(fingerprint_from_bytes(&canonicalize(json)?))
}

/// Fingerprint of a JSON value.
pub fn fingerprint_value(value: &Value) -> Result<Fingerprint, CanonError> {
    Ok(fingerprint_from_bytes(&canonicalize_value(value)?))
}

fn fingerprint_from_bytes(bytes: &[u8]) -> Fingerprint {
    Fingerprint {
        algorithm: ALGORITHM.to_owned(),
        canonicalization: CANONICALIZATION.to_owned(),
        value: sha256_hex(bytes),
    }
}

/// JSON numbers are IEEE 754 binary64 values, matching `JSON.parse` in
/// ECMAScript. `serde_json` keeps integers that fit in `u64` exact, including
/// integers that are not safe ES integers.
fn normalize(value: &Value) -> Result<Value, CanonError> {
    match value {
        Value::Number(number) => Ok(Value::Number(es_number(number)?)),
        Value::Array(items) => {
            let mut normalized = Vec::with_capacity(items.len());
            for item in items {
                normalized.push(normalize(item)?);
            }
            Ok(Value::Array(normalized))
        }
        Value::Object(entries) => {
            let mut normalized = Map::new();
            for (key, entry) in entries {
                normalized.insert(key.clone(), normalize(entry)?);
            }
            Ok(Value::Object(normalized))
        }
        Value::Null | Value::Bool(_) | Value::String(_) => Ok(value.clone()),
    }
}

fn es_number(number: &Number) -> Result<Number, CanonError> {
    let Some(value) = number.as_f64() else {
        return Err(CanonError::NonFiniteNumber);
    };
    if !value.is_finite() {
        return Err(CanonError::NonFiniteNumber);
    }
    Number::from_f64(value).ok_or(CanonError::NonFiniteNumber)
}

fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        encoded.push(HEX[(byte >> 4) as usize] as char);
        encoded.push(HEX[(byte & 0x0f) as usize] as char);
    }
    encoded
}

#[cfg(test)]
mod tests {
    use super::{canonicalize, fingerprint, sha256_hex, ALGORITHM, CANONICALIZATION};

    #[test]
    fn reordered_keys_hash_the_same_bytes() {
        let forward = canonicalize(r#"{"b":1,"a":2}"#).unwrap();
        let backward = canonicalize(r#"{"a":2,"b":1}"#).unwrap();
        let spaced = canonicalize(" { \"a\" : 2, \"b\" : 1 } ").unwrap();
        assert_eq!(forward, br#"{"a":2,"b":1}"#);
        assert_eq!(forward, backward);
        assert_eq!(forward, spaced);
        let hashed = fingerprint(r#"{"b":1,"a":2}"#).unwrap();
        assert_eq!(hashed.algorithm, ALGORITHM);
        assert_eq!(hashed.canonicalization, CANONICALIZATION);
        assert_eq!(
            hashed.value,
            "d3626ac30a87e6f7a6428233b3c68299976865fa5508e4267c5415c76af7a772"
        );
        assert_eq!(hashed.value, fingerprint(r#"{"a":2,"b":1}"#).unwrap().value);
        assert_eq!(hashed.value.len(), 64);
    }

    #[test]
    fn produced_fingerprint_matches_fingerprint_one_rules() {
        let record = serde_json::to_value(fingerprint(r#"{"sample":true}"#).unwrap()).unwrap();
        assert_eq!(record.as_object().unwrap().len(), 3);
        assert_eq!(record["algorithm"], "sha256");
        assert_eq!(record["canonicalization"], "rfc8785-jcs");
        let value = record["value"].as_str().unwrap();
        assert_eq!(value.len(), 64);
        assert!(value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)));
    }

    #[test]
    fn nested_keys_follow_the_same_order() {
        let bytes = canonicalize(r#"{"z":{"b":1,"a":2},"a":0}"#).unwrap();
        assert_eq!(bytes, br#"{"a":0,"z":{"a":2,"b":1}}"#);
        assert_eq!(bytes, canonicalize(r#"{"a":0,"z":{"a":2,"b":1}}"#).unwrap());
    }

    #[test]
    fn array_order_stays_put() {
        let forward = canonicalize(r#"[1,2]"#).unwrap();
        let backward = canonicalize(r#"[2,1]"#).unwrap();
        assert_eq!(forward, b"[1,2]");
        assert_ne!(forward, backward);
        assert_ne!(
            fingerprint(r#"[1,2]"#).unwrap().value,
            fingerprint(r#"[2,1]"#).unwrap().value
        );
    }

    #[test]
    fn supplementary_keys_sort_by_utf16_code_units() {
        let earlier_scalar = canonicalize("{\"\u{E000}\":1,\"\u{10000}\":2}").unwrap();
        let earlier_utf16 = canonicalize("{\"\u{10000}\":2,\"\u{E000}\":1}").unwrap();
        assert_eq!(earlier_scalar, earlier_utf16);
        let text = String::from_utf8(earlier_scalar).unwrap();
        let supplementary = text.find('\u{10000}').unwrap();
        let private_use = text.find('\u{E000}').unwrap();
        assert!(supplementary < private_use);
    }

    #[test]
    fn numbers_use_ecmascript_serialization() {
        assert_eq!(canonicalize("1.0").unwrap(), b"1");
        assert_eq!(canonicalize("1").unwrap(), b"1");
        assert_eq!(canonicalize("-0").unwrap(), b"0");
        assert_eq!(
            canonicalize("9007199254740993").unwrap(),
            canonicalize("9007199254740992").unwrap()
        );
        let tiny = canonicalize_bits(0x0000_0000_0000_0001);
        assert_eq!(tiny, b"5e-324");
    }

    fn canonicalize_bits(bits: u64) -> Vec<u8> {
        let value = serde_json::json!(f64::from_bits(bits));
        super::canonicalize_value(&value).unwrap()
    }

    #[test]
    fn sha256_of_empty_input_is_the_standard_vector() {
        assert_eq!(
            sha256_hex(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }
}
