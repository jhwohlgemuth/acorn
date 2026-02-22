//! Macros for logging and other utilities

/// Logging macro for failures
#[macro_export]
macro_rules! fail {
    ($msg:literal, $($rest:tt)*) => {
        tracing::error!(
            "{}",
            format!(
                "=> {} {}",
                $crate::util::Label::fail(),
                format!($msg, $($rest)*)
            )
        );
    };
    ($msg:literal) => {
        tracing::error!("{}", format!("=> {} {}", $crate::util::Label::fail(), $msg));
    };
    ($($args:tt)*) => {
        tracing::error!($($args)*);
    };
}
/// Logging macro for skipped operations
#[macro_export]
macro_rules! skip {
    ($msg:literal, $($rest:tt)*) => {
        tracing::warn!(
            "{}",
            format!(
                "=> {} {}",
                $crate::util::Label::skip(),
                format!($msg, $($rest)*)
            )
        );
    };
    ($msg:literal) => {
        tracing::warn!("{}", format!("=> {} {}", $crate::util::Label::skip(), $msg));
    };
    ($($args:tt)*) => {
        tracing::warn!($($args)*);
    };
}
/// Creates a `Param` with the given style, name, and values.
///
/// The macro supports two calling styles:
/// - **Ident style** (shorthand): `param!(QueryPair, "q", ...)`  
///   The style is automatically qualified to `ParamStyle::QueryPair`
/// - **Path style** (explicit): `param!(ParamStyle::QueryPair, "q", ...)`  
///   Use when the full path is preferred or when `ParamStyle` is in scope
///
/// # Value Syntaxes
///
/// The macro supports four different value syntaxes:
/// 1. Single value: `param!(FieldList, "fl", "family-name")`
/// 2. Single tuple: `param!(QueryPair, "filter", ("status", "inactive"))`
/// 3. Multiple tuples: `param!(QueryPair, "q", (("key1", "val1"), ("key2", "val2")))`
/// 4. Vec notation: `param!(FieldList, "fields", vec![vec!["field1"], vec!["field2"]])`
///
/// # Examples
///
/// ```ignore
/// // Shorthand with single value (ident style)
/// param!(FieldList, "fl", "family-name")
///
/// // Shorthand with single tuple (ident style)
/// param!(QueryPair, "filter", ("status", "inactive"))
///
/// // Shorthand with multiple tuples (ident style)
/// param!(QueryPair, "q", (
///     ("affiliation-org-name", "Lyrasis"),
///     ("ror-org-id", "\"https://ror.org/01qz5mb56\""),
/// ))
///
/// // Path style (explicit ParamStyle reference)
/// param!(ParamStyle::FieldList, "fl", "family-name")
///
/// // Vec notation
/// param!(QueryPair, "q", vec![
///     vec!["affiliation-org-name", "Lyrasis"],
/// ])
/// ```
#[macro_export]
macro_rules! param {
    // Ident with multiple tuples: param!(QueryPair, "q", (("a", "b"), ("c", "d")))
    ($style:ident, $name:expr, ( $( ($($val:expr),* $(,)?) ),+ $(,)? )) => {
        $crate::io::api::Param::of_type($crate::io::api::ParamStyle::$style)
            .values(vec![ $( vec![ $( Some($val) ),* ] ),* ])
            .with_key($name)
    };
    // Ident with single tuple: param!(QueryPair, "filter", ("status", "inactive"))
    ($style:ident, $name:expr, ($($val:expr),+ $(,)?)) => {
        $crate::io::api::Param::of_type($crate::io::api::ParamStyle::$style)
            .values(vec![vec![ $( Some($val) ),* ]])
            .with_key($name)
    };
    // Ident with vec notation: param!(FieldList, "fields", vec![vec!["f1"], vec!["f2"]])
    ($style:ident, $name:expr, vec![ $( vec![ $($val:expr),* $(,)? ] ),* $(,)? ]) => {
        $crate::io::api::Param::of_type($crate::io::api::ParamStyle::$style)
            .values(vec![ $( vec![ $( Some($val) ),* ] ),* ])
            .with_key($name)
    };
    // Ident with single value: param!(FieldList, "fl", "family-name")
    ($style:ident, $name:expr, $val:expr) => {
        $crate::io::api::Param::of_type($crate::io::api::ParamStyle::$style)
            .values(vec![vec![Some($val)]])
            .with_key($name)
    };
    // Path with multiple tuples: param!(ParamStyle::QueryPair, "q", (("a", "b"), ("c", "d")))
    ($style:path, $name:expr, ( $( ($($val:expr),* $(,)?) ),+ $(,)? )) => {
        $crate::io::api::Param::of_type($style)
            .values(vec![ $( vec![ $( Some($val) ),* ] ),* ])
            .with_key($name)
    };
    // Path with single tuple: param!(ParamStyle::QueryPair, "filter", ("status", "inactive"))
    ($style:path, $name:expr, ($($val:expr),+ $(,)?)) => {
        $crate::io::api::Param::of_type($style)
            .values(vec![vec![ $( Some($val) ),* ]])
            .with_key($name)
    };
    // Path with vec notation: param!(ParamStyle::FieldList, "fields", vec![vec!["f1"]])
    ($style:path, $name:expr, vec![ $( vec![ $($val:expr),* $(,)? ] ),* $(,)? ]) => {
        $crate::io::api::Param::of_type($style)
            .values(vec![ $( vec![ $( Some($val) ),* ] ),* ])
            .with_key($name)
    };
    // Path with single value: param!(ParamStyle::FieldList, "fl", "family-name")
    ($style:path, $name:expr, $val:expr) => {
        $crate::io::api::Param::of_type($style)
            .values(vec![vec![Some($val)]])
            .with_key($name)
    };
}
