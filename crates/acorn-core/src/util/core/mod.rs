//! Allocation-free utility functions
//!
/// Returns `true` if `a` and `b` have the same length and equal contents, without short-circuiting on the first differing byte.
pub fn constant_time_eq<A: AsRef<[u8]>, B: AsRef<[u8]>>(a: A, b: B) -> bool {
    let (a, b) = (a.as_ref(), b.as_ref());
    a.len() == b.len() && a.iter().zip(b.iter()).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}
/// Simple glob pattern matching supporting `*` (any sequence) and `?` (any single character).
pub fn glob_matches(path: &str, pattern: &str) -> bool {
    fn matches(path: &[u8], pattern: &[u8]) -> bool {
        match pattern.split_first() {
            | None => path.is_empty(),
            | Some((b'*', rest)) => matches(path, rest) || path.split_first().is_some_and(|(_, tail)| matches(tail, pattern)),
            | Some((b'?', rest)) => path.split_first().is_some_and(|(_, tail)| matches(tail, rest)),
            | Some((expected, rest)) => path.split_first().is_some_and(|(actual, tail)| expected == actual && matches(tail, rest)),
        }
    }
    matches(path.as_bytes(), pattern.as_bytes())
}
/// Returns `true` when both values are finite and `left` is greater than `right`.
pub fn is_finite_and_greater_than(left: f64, right: f64) -> bool {
    left.is_finite() && right.is_finite() && left > right
}
/// Return the longest consecutive run of `character` in `value`.
pub fn longest_character_run(value: &str, character: char) -> usize {
    value
        .chars()
        .fold((0usize, 0usize), |(longest, current), value| {
            let next = match value == character {
                | true => current.saturating_add(1),
                | false => 0,
            };
            (longest.max(next), next)
        })
        .0
}
/// Return the greatest value from an iterator.
pub fn maximum<T: Ord>(values: impl IntoIterator<Item = T>) -> Option<T> {
    values.into_iter().max()
}
/// Remove the first matching suffix from a string slice.
pub fn strip_suffixes<'a>(values: &[&str], name: &'a str) -> &'a str {
    values.iter().find_map(|value| name.strip_suffix(value)).unwrap_or(name)
}
/// Trim unmatched trailing `)` while preserving balanced parentheses in the string.
pub fn trim_unmatched_trailing_parentheses(value: &str) -> &str {
    let opening = value.chars().filter(|character| *character == '(').count();
    let closing = value.chars().filter(|character| *character == ')').count();
    let trailing = value.chars().rev().take_while(|character| *character == ')').count();
    let trim = closing.saturating_sub(opening).min(trailing);
    value.get(..value.len().saturating_sub(trim)).unwrap_or(value)
}

#[cfg(test)]
mod tests;
