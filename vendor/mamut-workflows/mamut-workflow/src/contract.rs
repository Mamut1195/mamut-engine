// SPDX-License-Identifier: Apache-2.0

//! Contract identifiers.
//!
//! The shared shape is `urn:mamut:<dominio>:<nombre>:<semver>`. This module
//! parses that shape so the compiler can compare majors. It is not the
//! `mamut-contracts` package.

use std::fmt;

use serde::de::{self, Deserializer, Visitor};
use serde::{Deserialize, Serialize, Serializer};

/// A contract id `urn:mamut:<dominio>:<nombre>:<semver>`.
///
/// `dominio` and `nombre` are lowercase ASCII letters, digits, and hyphens.
/// The version is `MAJOR.MINOR.PATCH` without a pre-release and without
/// leading zeros. Two ids match for substitution when domain, name, and major
/// match.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ContractId {
    domain: String,
    name: String,
    major: u64,
    minor: u64,
    patch: u64,
}

/// The string is not `urn:mamut:<dominio>:<nombre>:<semver>`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ContractIdError {
    /// The rejected text.
    pub value: String,
}

impl ContractId {
    /// Parse a contract id.
    pub fn parse(value: &str) -> Result<Self, ContractIdError> {
        let rest = value
            .strip_prefix("urn:mamut:")
            .ok_or_else(|| ContractIdError {
                value: value.to_owned(),
            })?;
        let mut parts = rest.split(':');
        let domain = parts.next();
        let name = parts.next();
        let version = parts.next();
        let extra = parts.next();
        let (Some(domain), Some(name), Some(version), None) = (domain, name, version, extra) else {
            return Err(ContractIdError {
                value: value.to_owned(),
            });
        };
        if !is_slug(domain) || !is_slug(name) {
            return Err(ContractIdError {
                value: value.to_owned(),
            });
        }
        let (major, minor, patch) = parse_semver(version).ok_or_else(|| ContractIdError {
            value: value.to_owned(),
        })?;
        Ok(Self {
            domain: domain.to_owned(),
            name: name.to_owned(),
            major,
            minor,
            patch,
        })
    }

    /// Canonical URN.
    pub fn urn(&self) -> String {
        format!(
            "urn:mamut:{}:{}:{}.{}.{}",
            self.domain, self.name, self.major, self.minor, self.patch
        )
    }

    /// Domain segment.
    pub fn domain(&self) -> &str {
        &self.domain
    }

    /// Name segment.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Major version.
    pub fn major(&self) -> u64 {
        self.major
    }

    /// Same domain, name, and major version.
    pub fn major_compatible(&self, other: &Self) -> bool {
        self.domain == other.domain && self.name == other.name && self.major == other.major
    }
}

impl fmt::Display for ContractId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.urn())
    }
}

impl fmt::Display for ContractIdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "contract id '{}' is not urn:mamut:<dominio>:<nombre>:<semver>",
            self.value
        )
    }
}

impl std::error::Error for ContractIdError {}

impl Serialize for ContractId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.urn())
    }
}

impl<'de> Deserialize<'de> for ContractId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ContractVisitor;

        impl Visitor<'_> for ContractVisitor {
            type Value = ContractId;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a urn:mamut:<dominio>:<nombre>:<semver> string")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                ContractId::parse(value).map_err(E::custom)
            }
        }

        deserializer.deserialize_str(ContractVisitor)
    }
}

fn is_slug(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !first.is_ascii_lowercase() {
        return false;
    }
    let mut last_was_hyphen = false;
    for ch in chars {
        if ch == '-' {
            last_was_hyphen = true;
            continue;
        }
        if !ch.is_ascii_lowercase() && !ch.is_ascii_digit() {
            return false;
        }
        last_was_hyphen = false;
    }
    !last_was_hyphen
}

fn parse_semver(value: &str) -> Option<(u64, u64, u64)> {
    let mut parts = value.split('.');
    let major = parse_component(parts.next()?)?;
    let minor = parse_component(parts.next()?)?;
    let patch = parse_component(parts.next()?)?;
    if parts.next().is_some() {
        return None;
    }
    Some((major, minor, patch))
}

fn parse_component(value: &str) -> Option<u64> {
    if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    if value.len() > 1 && value.starts_with('0') {
        return None;
    }
    value.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::ContractId;

    #[test]
    fn parses_the_owner_shape_and_compares_majors() {
        let current = ContractId::parse("urn:mamut:core:diagnostic:1.0.0").unwrap();
        let later = ContractId::parse("urn:mamut:core:diagnostic:1.2.0").unwrap();
        let next_major = ContractId::parse("urn:mamut:core:diagnostic:2.0.0").unwrap();
        assert_eq!(current.urn(), "urn:mamut:core:diagnostic:1.0.0");
        assert!(current.major_compatible(&later));
        assert!(!current.major_compatible(&next_major));
    }

    #[test]
    fn rejects_other_id_styles() {
        assert!(ContractId::parse("mamut.engine.model/1").is_err());
        assert!(ContractId::parse("urn:mamut:core:diagnostic:01.0.0").is_err());
        assert!(ContractId::parse("urn:mamut:Core:diagnostic:1.0.0").is_err());
        assert!(ContractId::parse("urn:mamut:steel-connections:connection-project:0.1.0").is_ok());
    }
}
