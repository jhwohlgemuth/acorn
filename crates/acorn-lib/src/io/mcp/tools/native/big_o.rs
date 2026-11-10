//! Read-only routed Big-O comparison backed by the portable core operation.
use super::{NativeCapability, NativeEffect, Register};
use crate::error::ApiResult;
use crate::io::mcp::ToolRegistry;
use acorn_core::big_o::{compare_notation, ComparisonInput, ComparisonOutput};

impl ToolRegistry {
    /// Register the offline-safe Big-O comparison capability and native binding.
    pub fn register_big_o_capability(self) -> ApiResult<Self> {
        self.register_native::<ComparisonInput, ComparisonOutput, _, _>(
            NativeCapability::new(
                "diff::bigo",
                "Compare two caller-supplied Big-O expressions under explicit or inferred growth assumptions.",
                NativeEffect::LocalRead,
            )
            .with_offline_support(),
            |input| async move { compare_notation(input).map_err(Into::into) },
        )
    }
}
