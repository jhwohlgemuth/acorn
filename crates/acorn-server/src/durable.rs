//! Durable-operation composition boundary.
//!
//! Concrete database persistence remains owned by the host application.
//! Server compositions may inject an [`acorn_service::registry::DurableOperations`] implementation when status and replay are available.

/// Marker for an intentionally unavailable durable adapter.
#[derive(Clone, Copy, Debug, Default)]
pub struct Unavailable;
