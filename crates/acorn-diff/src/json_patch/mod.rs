//! Deterministic [RFC 6902 JSON Patch](https://www.rfc-editor.org/rfc/rfc6902) primitives.
use alloc::{
    collections::BTreeSet,
    format,
    string::{String, ToString},
    vec::Vec,
};
use core::{cmp, fmt};
use serde::{Serialize, Serializer};
use serde_json::Value;

/// Computes deterministic patches between JSON values.
pub trait DiffValue {
    /// Compute a patch that transforms this value into `after`.
    fn diff(&self, after: &Self) -> JsonPatch;
}
trait DiffValueAt {
    fn diff_at(&self, after: &Value, path: &str, operations: &mut Vec<Operation>);
    fn diff_changed_at(&self, after: &Value, path: &str, operations: &mut Vec<Operation>);
}
/// Creates validated [RFC 6902](https://www.rfc-editor.org/rfc/rfc6902) operations.
pub trait Rfc6902: Sized {
    /// Creates an `add` operation.
    fn add(path: impl Into<String>, value: Value) -> Result<Self, PointerError>;
    /// Creates a `copy` operation.
    fn copy(from: impl Into<String>, path: impl Into<String>) -> Result<Self, PointerError>;
    /// Returns this operation's kind.
    fn kind(&self) -> OperationKind;
    /// Creates a `move` operation.
    fn move_(from: impl Into<String>, path: impl Into<String>) -> Result<Self, PointerError>;
    /// Creates a `remove` operation.
    fn remove(path: impl Into<String>) -> Result<Self, PointerError>;
    /// Creates a `replace` operation.
    fn replace(path: impl Into<String>, value: Value) -> Result<Self, PointerError>;
    /// Creates a `test` operation.
    fn test(path: impl Into<String>, value: Value) -> Result<Self, PointerError>;
}
/// The RFC 6902 operation represented by an [`Operation`].
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum OperationKind {
    /// Adds a value.
    Add,
    /// Copies a value.
    Copy,
    /// Moves a value.
    Move,
    /// Removes a value.
    Remove,
    /// Replaces a value.
    Replace,
    /// Tests a value.
    Test,
}
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "op", rename_all = "lowercase")]
enum OperationRepr {
    Add { path: String, value: Value },
    Copy { from: String, path: String },
    Move { from: String, path: String },
    Remove { path: String },
    Replace { path: String, value: Value },
    Test { path: String, value: Value },
}
/// An invalid JSON Pointer or move relationship.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PointerError {
    /// A pointer contains a `~` escape other than `~0` or `~1`.
    InvalidEscape(String),
    /// A pointer is not empty and does not begin with `/`.
    MissingLeadingSlash(String),
    /// A move destination is a child of its source.
    MoveIntoChild {
        /// Source pointer.
        from: String,
        /// Destination pointer.
        path: String,
    },
}
/// A deterministic sequence of RFC 6902 operations.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
#[serde(transparent)]
pub struct JsonPatch(Vec<Operation>);
/// A validated RFC 6902 operation.
#[derive(Clone, Debug, PartialEq)]
pub struct Operation(OperationRepr);
impl JsonPatch {
    /// Returns whether the patch has no operations.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    /// Returns the patch operations.
    #[must_use]
    pub fn operations(&self) -> &[Operation] {
        &self.0
    }
}
impl Rfc6902 for Operation {
    fn add(path: impl Into<String>, value: Value) -> Result<Self, PointerError> {
        validated(path.into()).map(|path| Self(OperationRepr::Add { path, value }))
    }
    fn copy(from: impl Into<String>, path: impl Into<String>) -> Result<Self, PointerError> {
        validated(from.into()).and_then(|from| validated(path.into()).map(|path| Self(OperationRepr::Copy { from, path })))
    }
    fn kind(&self) -> OperationKind {
        OperationKind::from(self.0.clone())
    }
    fn move_(from: impl Into<String>, path: impl Into<String>) -> Result<Self, PointerError> {
        validated(from.into()).and_then(|from| {
            validated(path.into()).and_then(|path| match is_proper_prefix(&from, &path) {
                | true => Err(PointerError::MoveIntoChild { from, path }),
                | false => Ok(Self(OperationRepr::Move { from, path })),
            })
        })
    }
    fn remove(path: impl Into<String>) -> Result<Self, PointerError> {
        validated(path.into()).map(|path| Self(OperationRepr::Remove { path }))
    }
    fn replace(path: impl Into<String>, value: Value) -> Result<Self, PointerError> {
        validated(path.into()).map(|path| Self(OperationRepr::Replace { path, value }))
    }
    fn test(path: impl Into<String>, value: Value) -> Result<Self, PointerError> {
        validated(path.into()).map(|path| Self(OperationRepr::Test { path, value }))
    }
}
impl Serialize for Operation {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.serialize(serializer)
    }
}
impl From<OperationRepr> for OperationKind {
    fn from(value: OperationRepr) -> Self {
        match value {
            | OperationRepr::Add { .. } => Self::Add,
            | OperationRepr::Copy { .. } => Self::Copy,
            | OperationRepr::Move { .. } => Self::Move,
            | OperationRepr::Remove { .. } => Self::Remove,
            | OperationRepr::Replace { .. } => Self::Replace,
            | OperationRepr::Test { .. } => Self::Test,
        }
    }
}
impl core::error::Error for PointerError {}
impl fmt::Display for PointerError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            | Self::InvalidEscape(pointer) => write!(formatter, "JSON Pointer contains an invalid '~' escape: {pointer}"),
            | Self::MissingLeadingSlash(pointer) => write!(formatter, "JSON Pointer must be empty or begin with '/': {pointer}"),
            | Self::MoveIntoChild { from, path } => write!(formatter, "cannot move {from} into its child {path}"),
        }
    }
}
impl DiffValue for Value {
    fn diff(&self, after: &Self) -> JsonPatch {
        let mut operations = Vec::new();
        self.diff_at(after, "", &mut operations);
        JsonPatch(operations)
    }
}
impl DiffValueAt for Value {
    fn diff_at(&self, after: &Value, path: &str, operations: &mut Vec<Operation>) {
        match self == after {
            | true => {}
            | false => self.diff_changed_at(after, path, operations),
        }
    }
    fn diff_changed_at(&self, after: &Value, path: &str, operations: &mut Vec<Operation>) {
        match (self, after) {
            | (Value::Object(before), Value::Object(after)) => {
                let before_keys = before.keys().collect::<BTreeSet<_>>();
                let after_keys = after.keys().collect::<BTreeSet<_>>();
                before_keys.difference(&after_keys).for_each(|key| {
                    operations.push(Operation(OperationRepr::Remove { path: join(path, key) }));
                });
                before_keys.intersection(&after_keys).for_each(|key| {
                    let before_value = &before[*key];
                    let after_value = &after[*key];
                    match before_value == after_value {
                        | true => {}
                        | false => before_value.diff_changed_at(after_value, &join(path, key), operations),
                    }
                });
                after_keys.difference(&before_keys).for_each(|key| {
                    operations.push(Operation(OperationRepr::Add {
                        path: join(path, key),
                        value: after[*key].clone(),
                    }));
                });
            }
            | (Value::Array(before), Value::Array(after)) => {
                let shared = cmp::min(before.len(), after.len());
                before
                    .iter()
                    .zip(after)
                    .take(shared)
                    .enumerate()
                    .for_each(|(index, (before, after))| match before == after {
                        | true => {}
                        | false => before.diff_changed_at(after, &format!("{path}/{index}"), operations),
                    });
                (after.len()..before.len()).rev().for_each(|index| {
                    operations.push(Operation(OperationRepr::Remove {
                        path: format!("{path}/{index}"),
                    }));
                });
                after.iter().skip(shared).for_each(|value| {
                    operations.push(Operation(OperationRepr::Add {
                        path: format!("{path}/-"),
                        value: value.clone(),
                    }));
                });
            }
            | _ => operations.push(Operation(OperationRepr::Replace {
                path: path.to_string(),
                value: after.clone(),
            })),
        }
    }
}
fn has_invalid_escape(pointer: &str) -> bool {
    pointer
        .as_bytes()
        .iter()
        .enumerate()
        .any(|(index, character)| *character == b'~' && !matches!(pointer.as_bytes().get(index.saturating_add(1)), Some(b'0' | b'1')))
}
fn is_proper_prefix(from: &str, path: &str) -> bool {
    path.len() > from.len() && path.starts_with(from) && path.as_bytes().get(from.len()).copied() == Some(b'/')
}
fn join(path: &str, token: &str) -> String {
    fn escape(token: &str) -> String {
        token.replace('~', "~0").replace('/', "~1")
    }
    format!("{path}/{}", escape(token))
}
fn validated(pointer: String) -> Result<String, PointerError> {
    match (!pointer.is_empty() && !pointer.starts_with('/'), has_invalid_escape(&pointer)) {
        | (true, _) => Err(PointerError::MissingLeadingSlash(pointer)),
        | (_, true) => Err(PointerError::InvalidEscape(pointer)),
        | _ => Ok(pointer),
    }
}

#[cfg(test)]
mod tests;
