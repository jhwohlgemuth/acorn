//! Database schema model
//!
use serde::{Deserialize, Serialize};

/// Database tables
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) enum Table {
    /// Maintain CLI activity log
    Activity,
    /// Stores research activity index entries downloaded from buckets
    Catalog,
}
