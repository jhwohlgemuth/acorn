//! Export projections organized by source standard.

mod plan;
pub use plan::{bag_destination, pdf_destination};
#[cfg(feature = "pdf")]
pub mod pdf;
#[cfg(feature = "powerpoint")]
pub mod powerpoint;
mod rads;
pub use rads::{ExportContext, ImageSource};
mod schema;
pub use schema::StandardSchema;
pub mod updates;

#[cfg(test)]
mod tests;
