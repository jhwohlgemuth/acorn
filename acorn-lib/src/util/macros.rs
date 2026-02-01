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
