use super::super::{NativeCapability, NativeEffect, Register};
use super::{
    ApiResult, ExportBagInput, ExportBagOutput, ExportBatchInput, ExportBatchOutput, ExportCrosswalkInput, ExportDocumentInput, ExportDocumentOutput,
    ExportPreviewInput, ExportPreviewOutput, ExportRenderInput, ExportRenderOutput, ExportRenderServices, ExportSchemaInput, ExportSchemaOutput,
    ExportUpdatesPreviewInput, ToolRegistry,
};
use std::path::Path;

impl ToolRegistry {
    /// Register exact-authority BagIt archive export.
    pub fn register_export_bag_capability(self, root: impl AsRef<Path>) -> ApiResult<Self> {
        let root = root.as_ref().to_path_buf();
        self.register_native::<ExportBagInput, ExportBagOutput, _, _>(
            NativeCapability::new(
                "export::bag",
                "Archive a confirmed bounded local directory as BagIt.",
                NativeEffect::LocalWrite,
            )
            .with_offline_support(),
            move |input| {
                let root = root.clone();
                async move { input.execute(&root) }
            },
        )
    }
    /// Register exact-authority JSONL batch export.
    pub fn register_export_batch_capability(self, root: impl AsRef<Path>) -> ApiResult<Self> {
        let root = root.as_ref().to_path_buf();
        self.register_native::<ExportBatchInput, ExportBatchOutput, _, _>(
            NativeCapability::new(
                "export::batch",
                "Write one confirmed local RAD batch as a JSONL artifact.",
                NativeEffect::LocalWrite,
            )
            .with_offline_support(),
            move |input| {
                let root = root.clone();
                async move { input.execute(&root) }
            },
        )
    }
    /// Register explicit structured standard crosswalk export.
    pub fn register_export_crosswalk_capability(self, root: impl AsRef<Path>) -> ApiResult<Self> {
        let root = root.as_ref().to_path_buf();
        self.register_native::<ExportCrosswalkInput, ExportBatchOutput, _, _>(
            NativeCapability::new(
                "export::crosswalk",
                "Write one explicit structured standard crosswalk to an exact local artifact.",
                NativeEffect::LocalWrite,
            )
            .with_offline_support(),
            move |input| {
                let root = root.clone();
                async move { input.execute(&root) }
            },
        )
    }
    /// Register exact-authority RAD text document export.
    pub fn register_export_document_capability(self, root: impl AsRef<Path>) -> ApiResult<Self> {
        let root = root.as_ref().to_path_buf();
        self.register_native::<ExportDocumentInput, ExportDocumentOutput, _, _>(
            NativeCapability::new(
                "export::document",
                "Write a confirmed local RAD document as CFF, JSON, Markdown, YAML, or ZON.",
                NativeEffect::LocalWrite,
            )
            .with_offline_support(),
            move |input| {
                let root = root.clone();
                async move { input.execute(&root) }
            },
        )
    }
    /// Register read-only export planning for confirmed local sources.
    pub fn register_export_preview_capability(self, root: impl AsRef<Path>) -> ApiResult<Self> {
        let root = root.as_ref().to_path_buf();
        self.register_native::<ExportPreviewInput, ExportPreviewOutput, _, _>(
            NativeCapability::new(
                "export::preview",
                "Preview one bounded local archive, document, graph, or updates export without writing.",
                NativeEffect::LocalRead,
            )
            .with_offline_support(),
            move |input| {
                let root = root.clone();
                async move { input.execute(&root) }
            },
        )
    }
    /// Register exact-authority local PDF, PowerPoint, and updates rendering.
    pub fn register_export_render_capability(self, root: impl AsRef<Path>, offline: bool, services: ExportRenderServices) -> ApiResult<Self> {
        let root = root.as_ref().to_path_buf();
        let capability = NativeCapability::new(
            "export::render",
            "Render one confirmed RAD source to an exact PDF, PowerPoint, or updates artifact.",
            NativeEffect::RemoteWrite,
        );
        let capability = match offline {
            | true => capability.with_offline_support(),
            | false => capability,
        };
        self.register_native::<ExportRenderInput, ExportRenderOutput, _, _>(capability, move |input| {
            let root = root.clone();
            let services = services.clone();
            async move { input.execute(&root, offline, &services).await }
        })
    }
    /// Register read-only schema export.
    pub fn register_export_schema_capability(self) -> ApiResult<Self> {
        self.register_native::<ExportSchemaInput, ExportSchemaOutput, _, _>(
            NativeCapability::new(
                "export::schema",
                "Render a modeled metadata schema as JSON or YAML text.",
                NativeEffect::LocalRead,
            )
            .with_offline_support(),
            |input| async move { input.execute() },
        )
    }
    /// Register read-only since-filtered updates export planning.
    pub fn register_export_updates_preview_capability(self, root: impl AsRef<Path>) -> ApiResult<Self> {
        let root = root.as_ref().to_path_buf();
        self.register_native::<ExportUpdatesPreviewInput, ExportPreviewOutput, _, _>(
            NativeCapability::new(
                "export::updates::preview",
                "Preview one since-filtered Markdown, DOCX, or PDF updates artifact without writing.",
                NativeEffect::LocalRead,
            )
            .with_offline_support(),
            move |input| {
                let root = root.clone();
                async move { input.execute(&root) }
            },
        )
    }
}
