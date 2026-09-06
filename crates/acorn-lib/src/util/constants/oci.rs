//! OCI, ModelKit, ModelPack, and ORAS constants.
/// ACORN-specific OCI layer media type for model files.
pub const ACORN_MODEL_LAYER: &str = "application/vnd.acorn.model.file.v1";
/// Docker image configuration media type used to identify runnable images.
pub const DOCKER_IMAGE_CONFIG: &str = "application/vnd.docker.container.image.v1+json";
/// Docker manifest-list media type accepted for model artifact indirection.
pub const DOCKER_MANIFEST_LIST: &str = "application/vnd.docker.distribution.manifest.list.v2+json";
/// KitOps ModelKit configuration media type.
pub const KITOPS_MODELKIT_CONFIG: &str = "application/vnd.kitops.modelkit.config.v1+json";
/// Verbatim KitOps ModelKit MCP Bundle layer media type.
pub const KITOPS_MODELKIT_MCPB_RAW: &str = "application/vnd.kitops.modelkit.mcpb.v1.raw";
/// Gzip-compressed KitOps ModelKit model layer media type.
pub const KITOPS_MODELKIT_MODEL_GZIP: &str = "application/vnd.kitops.modelkit.model.v1.tar+gzip";
/// Gzip-compressed KitOps ModelKit model-part layer media type.
pub const KITOPS_MODELKIT_MODEL_PART_GZIP: &str = "application/vnd.kitops.modelkit.modelpart.v1.tar+gzip";
/// Minimum supported ORAS CLI version.
pub const MINIMUM_ORAS_VERSION: (u64, u64, u64) = (1, 3, 0);
/// Standard manifest filename for ACORN-compatible ModelKits.
pub const MODELKIT_MANIFEST: &str = "needle-modelkit.json";
/// CNCF ModelPack raw code layer media type.
pub const MODELPACK_CODE_RAW: &str = "application/vnd.cncf.model.code.v1.raw";
/// CNCF ModelPack TAR code layer media type.
pub const MODELPACK_CODE_TAR: &str = "application/vnd.cncf.model.code.v1.tar";
/// CNCF ModelPack gzip TAR code layer media type.
pub const MODELPACK_CODE_TAR_GZIP: &str = "application/vnd.cncf.model.code.v1.tar+gzip";
/// CNCF ModelPack Zstandard TAR code layer media type.
pub const MODELPACK_CODE_TAR_ZSTD: &str = "application/vnd.cncf.model.code.v1.tar+zstd";
/// CNCF ModelPack configuration media type.
pub const MODELPACK_CONFIG: &str = "application/vnd.cncf.model.config.v1+json";
/// CNCF ModelPack raw dataset layer media type.
pub const MODELPACK_DATASET_RAW: &str = "application/vnd.cncf.model.dataset.v1.raw";
/// CNCF ModelPack TAR dataset layer media type.
pub const MODELPACK_DATASET_TAR: &str = "application/vnd.cncf.model.dataset.v1.tar";
/// CNCF ModelPack gzip TAR dataset layer media type.
pub const MODELPACK_DATASET_TAR_GZIP: &str = "application/vnd.cncf.model.dataset.v1.tar+gzip";
/// CNCF ModelPack Zstandard TAR dataset layer media type.
pub const MODELPACK_DATASET_TAR_ZSTD: &str = "application/vnd.cncf.model.dataset.v1.tar+zstd";
/// CNCF ModelPack raw documentation layer media type.
pub const MODELPACK_DOC_RAW: &str = "application/vnd.cncf.model.doc.v1.raw";
/// CNCF ModelPack TAR documentation layer media type.
pub const MODELPACK_DOC_TAR: &str = "application/vnd.cncf.model.doc.v1.tar";
/// CNCF ModelPack gzip TAR documentation layer media type.
pub const MODELPACK_DOC_TAR_GZIP: &str = "application/vnd.cncf.model.doc.v1.tar+gzip";
/// CNCF ModelPack Zstandard TAR documentation layer media type.
pub const MODELPACK_DOC_TAR_ZSTD: &str = "application/vnd.cncf.model.doc.v1.tar+zstd";
/// CNCF ModelPack artifact type.
pub const MODELPACK_MANIFEST: &str = "application/vnd.cncf.model.manifest.v1+json";
/// CNCF ModelPack raw weight-configuration layer media type.
pub const MODELPACK_WEIGHT_CONFIG_RAW: &str = "application/vnd.cncf.model.weight.config.v1.raw";
/// CNCF ModelPack TAR weight-configuration layer media type.
pub const MODELPACK_WEIGHT_CONFIG_TAR: &str = "application/vnd.cncf.model.weight.config.v1.tar";
/// CNCF ModelPack gzip TAR weight-configuration layer media type.
pub const MODELPACK_WEIGHT_CONFIG_TAR_GZIP: &str = "application/vnd.cncf.model.weight.config.v1.tar+gzip";
/// CNCF ModelPack Zstandard TAR weight-configuration layer media type.
pub const MODELPACK_WEIGHT_CONFIG_TAR_ZSTD: &str = "application/vnd.cncf.model.weight.config.v1.tar+zstd";
/// Prefix shared by CNCF ModelPack model-weight layer media types.
pub const MODELPACK_WEIGHT_PREFIX: &str = "application/vnd.cncf.model.weight.v1.";
/// CNCF ModelPack raw model-weight layer media type.
pub const MODELPACK_WEIGHT_RAW: &str = "application/vnd.cncf.model.weight.v1.raw";
/// CNCF ModelPack uncompressed TAR model-weight layer media type.
pub const MODELPACK_WEIGHT_TAR: &str = "application/vnd.cncf.model.weight.v1.tar";
/// CNCF ModelPack gzip-compressed TAR model-weight layer media type.
pub const MODELPACK_WEIGHT_TAR_GZIP: &str = "application/vnd.cncf.model.weight.v1.tar+gzip";
/// CNCF ModelPack zstd-compressed TAR model-weight layer media type.
pub const MODELPACK_WEIGHT_TAR_ZSTD: &str = "application/vnd.cncf.model.weight.v1.tar+zstd";
/// OCI image configuration media type used to identify runnable images.
pub const OCI_IMAGE_CONFIG: &str = "application/vnd.oci.image.config.v1+json";
/// OCI image index media type accepted for model artifact indirection.
pub const OCI_IMAGE_INDEX: &str = "application/vnd.oci.image.index.v1+json";
/// OCI image manifest media type accepted for model artifacts.
pub const OCI_IMAGE_MANIFEST: &str = "application/vnd.oci.image.manifest.v1+json";
/// Uncompressed OCI image layer media type accepted for model files.
pub const OCI_LAYER: &str = "application/vnd.oci.image.layer.v1.tar";
/// Compressed OCI image layer media type accepted for model files.
pub const OCI_LAYER_GZIP: &str = "application/vnd.oci.image.layer.v1.tar+gzip";
/// Explicit URI prefix used to route model references through OCI.
pub const OCI_PREFIX: &str = "oci://";
/// Millisecond delays used when retrying a temporarily busy ORAS executable.
pub const ORAS_EXECUTABLE_BUSY_RETRY_DELAYS_MILLISECONDS: [u64; 3] = [10, 25, 50];
