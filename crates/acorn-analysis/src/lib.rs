//! Portable ACORN analysis models and supplied-content checks.
#![cfg_attr(not(feature = "std"), no_std)]
extern crate alloc;

pub mod check;
mod constants;
pub mod error;
pub mod fair;
pub mod readability;
pub mod vale;

pub use check::{
    check_readability_for_text, Check, CheckBatch, CheckCategory, CheckKind, CheckSeverity, CheckSummary, Finding, Findings,
    StandardCheckCapabilities, VocabularyAdvisory, VocabularyAdvisoryCandidate,
};
#[cfg(feature = "analysis")]
pub use check::{IntoChecks, StandardCrosswalk};
pub use fair::{
    FairCounts, FairDetails, FairEvidence, FairEvidenceItem, FairIndicator, FairIndicatorId, FairPrinciple, FairPriority, FairProfile, FairState,
};

/// Build an analyzer [`Check`] with required category and success values.
#[macro_export]
macro_rules! check {
    ($category:expr, $success:expr $(, $field:ident : $value:expr )* $(,)?) => {
        $crate::check!(@apply $crate::Check::init().category($category).success($success) $(, $field : $value )*)
    };
    (@apply $builder:expr) => {
        $builder.build()
    };
    (@apply $builder:expr, $field:ident : $value:expr $(, $rest_field:ident : $rest_value:expr )* ) => {
        $crate::check!(@apply $builder.$field($value) $(, $rest_field : $rest_value )*)
    };
}
/// Build a failing analyzer [`Check`] with error severity.
#[macro_export]
macro_rules! check_err {
    ($category:expr $(, $field:ident : $value:expr )* $(,)?) => {
        $crate::check!($category, false, severity: $crate::CheckSeverity::Error $(, $field : $value )*)
    };
}
/// Build a successful analyzer [`Check`].
#[macro_export]
macro_rules! check_ok {
    ($category:expr $(, $field:ident : $value:expr )* $(,)?) => {
        $crate::check!($category, true $(, $field : $value )*)
    };
}

#[cfg(test)]
mod tests;
