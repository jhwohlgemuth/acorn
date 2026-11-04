//! Export projections organized by source standard.

mod research_activity;
pub use research_activity::{ExportContext, ImageSource};
mod plan;
pub use plan::{bag_destination, pdf_destination};
#[cfg(feature = "pdf")]
pub mod pdf;
#[cfg(feature = "powerpoint")]
pub mod powerpoint;
mod schema;
pub use schema::StandardSchema;
pub mod updates;

#[cfg(test)]
mod tests;
