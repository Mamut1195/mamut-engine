// SPDX-License-Identifier: AGPL-3.0-only

//! Natural byte order for textual identifiers (`cmp_natural`).
//!
//! Digit runs are compared by stripped length and raw bytes. A run is never
//! parsed as an integer. U7 stays open: the comparator accepts the long case,
//! and the public converter still rejects identifiers longer than 256 bytes.

use std::cmp::Ordering;

/// Owned comparison key. Built once and reused with `sort_by_cached_key`.
#[derive(Clone, Eq, PartialEq)]
pub(crate) struct NaturalKey {
    /// `0` when the id contains an ASCII digit, otherwise `1`.
    group: u8,
    runs: Vec<Run>,
    original: Vec<u8>,
}

#[derive(Clone, Eq, PartialEq)]
enum Run {
    Digits(Vec<u8>),
    Text(Vec<u8>),
}

impl PartialOrd for Run {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Run {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Self::Digits(left), Self::Digits(right)) => {
                left.len().cmp(&right.len()).then_with(|| left.cmp(right))
            }
            (Self::Digits(_), Self::Text(_)) => Ordering::Less,
            (Self::Text(_), Self::Digits(_)) => Ordering::Greater,
            (Self::Text(left), Self::Text(right)) => left.cmp(right),
        }
    }
}

impl PartialOrd for NaturalKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for NaturalKey {
    fn cmp(&self, other: &Self) -> Ordering {
        self.group
            .cmp(&other.group)
            .then_with(|| self.runs.cmp(&other.runs))
            .then_with(|| self.original.cmp(&other.original))
    }
}

/// Precompute the natural-order key of one identifier.
pub(crate) fn natural_key(id: &[u8]) -> NaturalKey {
    let mut runs = Vec::new();
    let mut index = 0;
    let mut any_digit = false;
    while index < id.len() {
        let digit = id[index].is_ascii_digit();
        let start = index;
        index += 1;
        while index < id.len() && id[index].is_ascii_digit() == digit {
            index += 1;
        }
        if digit {
            any_digit = true;
            runs.push(Run::Digits(strip_ascii_zeros(&id[start..index])));
        } else {
            runs.push(Run::Text(id[start..index].to_vec()));
        }
    }
    NaturalKey {
        group: if any_digit { 0 } else { 1 },
        runs,
        original: id.to_vec(),
    }
}

/// Compare two identifiers with `cmp_natural`.
///
/// Sorted collections use [`natural_key`] once per id. This function stays
/// available for a single comparison; production sorts do not call it.
#[allow(dead_code)]
pub(crate) fn cmp_natural(left: &[u8], right: &[u8]) -> Ordering {
    natural_key(left).cmp(&natural_key(right))
}

fn strip_ascii_zeros(digits: &[u8]) -> Vec<u8> {
    match digits.iter().position(|byte| *byte != b'0') {
        Some(start) => digits[start..].to_vec(),
        None => vec![b'0'],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cmp(left: &str, right: &str) -> Ordering {
        cmp_natural(left.as_bytes(), right.as_bytes())
    }

    fn less(left: &str, right: &str) {
        assert_eq!(cmp(left, right), Ordering::Less, "{left:?} < {right:?}");
        assert_eq!(cmp(right, left), Ordering::Greater, "{right:?} > {left:?}");
    }

    #[test]
    fn normative_example_and_cached_key_sort() {
        let expected = ["0", "1", "10", "-1", "S1", "S2", "S10", "S-1", "", "A", "S"];
        let mut ids = vec!["S", "A", "", "S-1", "S10", "S2", "S1", "-1", "10", "1", "0"];
        ids.sort_by_cached_key(|id| natural_key(id.as_bytes()));
        assert_eq!(ids, expected);
    }

    #[test]
    fn v42_cases() {
        less("S1", "S2");
        less("S2", "S10");
        less("S1", "S10");
        less("N2", "N10");
        less("a1b2", "a1b10");
        less("a1b10", "a2b1");
        less("S01", "S1");
        less("0", "00");
        less("0", "1");
        less("007", "8");
        less("99999999999999999999999", "100000000000000000000000");
        less("S99999999999999999999999", "S100000000000000000000000");
        less("S1", "S");
        less("S1", "S1a");
        less("S1", "S1-2");
        less("S1a", "S1b");
        less("A1", "a1");
        less("base", "top");
        less("Z", "a");
        less("z9", "a");
        let roman = "Ⅻ";
        let arabic = "٣";
        assert_eq!(natural_key(roman.as_bytes()).group, 1);
        assert_eq!(natural_key(arabic.as_bytes()).group, 1);
        less("0", roman);
        less("0", arabic);
        less("N1", &format!("N{arabic}"));
        less("n", "ñ");
        less("e\u{0301}", "\u{00e9}");
        less("🏗1", "🏗2");
        less("🏗2", "🏗10");
        less("1a", "a1");
        less("S1", "Sa");
        less("", "a");
        less("1", "");
        assert_eq!(cmp("S1", "S1"), Ordering::Equal);
        let nine = format!("S{}", "9".repeat(4096));
        let zeros = format!("S1{}", "0".repeat(4096));
        less(&nine, &zeros);
        less(&format!("S{}5", "0".repeat(100)), "S5");
        less("x10y", "x10y2");
        // Case 36 asks for both directions. `less` already checks the reverse.
        less("S2", "S10");
    }

    #[test]
    fn sample_is_totally_ordered() {
        let ids = ["0", "1", "10", "-1", "S01", "S1", "00", "S", "", "🏗10", "ñ"];
        for left in &ids {
            assert_eq!(cmp(left, left), Ordering::Equal);
            for middle in &ids {
                for right in &ids {
                    if cmp(left, middle) == Ordering::Less && cmp(middle, right) == Ordering::Less {
                        assert_eq!(cmp(left, right), Ordering::Less);
                    }
                }
            }
        }
    }
}
