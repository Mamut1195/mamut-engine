//! Deterministic context for a broken reference.
//!
//! The stable code stays `ENG-E-0003`. These types only carry the missing
//! entity, the path that cites it, and nearby identifiers.

/// Identifier sought by a reference.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MissingId {
    /// Decimal identifier, without leading zeros.
    Number(u64),
    /// Name, kept as written.
    Name(String),
}

impl MissingId {
    /// Canonical text used in the message and in [`similar_ids`].
    pub fn text(&self) -> String {
        match self {
            Self::Number(id) => id.to_string(),
            Self::Name(id) => id.clone(),
        }
    }
}

/// Extra context on a broken reference. Other diagnostics leave it empty.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReferenceContext {
    /// Kind of the missing entity, for example `node` or `section`.
    pub entity_kind: &'static str,
    /// Identifier that was sought.
    pub missing_id: MissingId,
    /// Path of the citation. Each index is the 0-based position in that
    /// model collection, not the entity id. `members[3].nodes[1]` is the
    /// second end of the member stored at position 3.
    pub referenced_by: String,
    /// Nearby ids of the same kind, already ordered by [`similar_ids`].
    pub suggestions: Vec<String>,
}

/// Suggestions for a missing identifier.
///
/// The cutoff and the order do not depend on locale, clocks, or map iteration:
///
/// 1. Each id is canonical text. An integer uses decimal digits without
///    leading zeros. A name is the exact sequence of Unicode scalars.
/// 2. Distance is Levenshtein on those scalars. Insertion, deletion, and
///    substitution each cost 1. A transposition is not its own edit, so
///    `ab` against `ba` costs 2.
/// 3. An existing id is kept when the distance is greater than 0 and at most
///    `max(1, missing_length / 2)`, using integer division of the sought id.
/// 4. Sort by distance, then by the UTF-8 bytes of the canonical text.
///    The same text is returned once.
/// 5. At most three ids are returned. No accepted id yields an empty list.
///
/// ```
/// use engine_core::similar_ids;
/// assert_eq!(similar_ids("99", ["98", "100", "1"]), vec!["98".to_string()]);
/// assert!(similar_ids("99", ["1", "2"]).is_empty());
/// assert!(similar_ids("ab", ["ba"]).is_empty());
/// ```
pub fn similar_ids<'a>(missing: &str, existing: impl IntoIterator<Item = &'a str>) -> Vec<String> {
    let limit = 1.max(missing.chars().count() / 2);
    let mut found = Vec::new();
    for candidate in existing {
        let distance = levenshtein(missing, candidate);
        if distance > 0 && distance <= limit {
            found.push((distance, candidate));
        }
    }
    found.sort_by(|left, right| left.0.cmp(&right.0).then_with(|| left.1.cmp(right.1)));
    found.dedup_by(|left, right| left.1 == right.1);
    found
        .into_iter()
        .take(3)
        .map(|(_, text)| text.to_owned())
        .collect()
}

fn levenshtein(left: &str, right: &str) -> usize {
    let left: Vec<char> = left.chars().collect();
    let right: Vec<char> = right.chars().collect();
    let mut previous: Vec<usize> = (0..=right.len()).collect();
    let mut current = vec![0; right.len() + 1];
    for (row, left_char) in left.iter().enumerate() {
        current[0] = row + 1;
        for (column, right_char) in right.iter().enumerate() {
            let substitution = previous[column] + usize::from(left_char != right_char);
            let insertion = current[column] + 1;
            let deletion = previous[column + 1] + 1;
            current[column + 1] = deletion.min(insertion).min(substitution);
        }
        std::mem::swap(&mut previous, &mut current);
    }
    previous[right.len()]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nearby_ids_follow_distance_then_text_and_stop_at_three() {
        assert_eq!(
            similar_ids("cat", ["cut", "cot", "car", "cab", "dog"]),
            vec!["cab".to_string(), "car".to_string(), "cot".to_string()]
        );
    }

    #[test]
    fn repeated_text_is_listed_once_and_the_order_does_not_change() {
        let once = similar_ids("99", ["98", "98", "97"]);
        assert_eq!(once, similar_ids("99", ["97", "98"]));
        assert_eq!(once, vec!["97".to_string(), "98".to_string()]);
    }

    #[test]
    fn a_short_id_does_not_accept_a_distant_neighbor() {
        assert!(similar_ids("99", ["1", "2", "100"]).is_empty());
        assert_eq!(similar_ids("", ["a"]), vec!["a".to_string()]);
    }
}
