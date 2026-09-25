---
okf_version: "0.1"
type: Plan
title: Provider-neutral bucket source abstraction
description: Separate bucket source discovery and retrieval from shared transfer processing while preserving compatibility and adding concise bucket definitions.
difficulty: medium
impact: high
package:
  - acorn-lib
  - acorn-cli
tags:
  - bucket
  - provider
  - refactor
  - transfer
status: planned
---

# Project code boundaries

- **Update:** `crates/acorn-lib/src/io/config/{bucket,mod,transfer}.rs`; focused configuration, transfer, manifest, and provenance tests under `crates/acorn-lib/src/io/config/tests/`; production import and download code and tests under `crates/acorn-cli/src/commands/{import,download}/`; configuration and download documentation.
- **Create:** `crates/acorn-lib/src/io/config/bucket_source.rs`; dedicated compatibility fixtures under `tests/fixtures/config/` only when existing fixtures cannot express the required aliases and conflicts.
- **Delete:** None.

# Dependencies

This plan consumes the manifest, filtering, destination-safety, canonicalization, provenance, and ingestion pipeline established by [unified import and download with BagIt archives](../features/unified-import-download-bagit.md). It must preserve [dependency feature defaults](portability/dependency-feature-defaults.md), the existing HTTP and offline policy, and the current public behavior of repository-backed buckets.

This plan is a prerequisite of [Azure Cosmos DB bucket downloads](../features/azure-cosmos-buckets.md). The Cosmos plan may add one source variant and provider adapter only after the source type, item contract, legacy configuration compatibility, and repository adapter in this plan have landed.

[JSON Lines recognition and export crosswalk](../features/json-lines-export-crosswalk.md) is independent. A later JSON Lines bucket proposal must consume this abstraction and the shared transfer pipeline rather than extend `acorn_core::Repository` or create another transfer executor.

# Goal

Give `Bucket` a bucket-specific source type and a provider-neutral discovery and retrieval contract. Make common buckets concise to author in configuration and code while adapting all existing repository-backed behavior without changing supported repositories, existing configuration files, selected paths, transfer manifests, provenance, CLI output, or local-database ingestion.

# Current evidence

- `Bucket` currently stores `acorn_core::Repository` directly as `code_repository`, with `repository` accepted as a serde alias.
- `Bucket::copy_files` and `Bucket::download_files` combine provider discovery and retrieval with shared filtering, flattening, destination planning, concurrent transfer, and manifest construction.
- Repository-specific branch resolution, raw-URL construction, authentication, and GitLab fallback behavior are implemented inside `bucket.rs`, so another bucket provider would have to duplicate orchestration or add unrelated variants to `acorn_core::Repository`.
- `TransferManifest` already owns transferred paths and drives canonicalization, provenance, and ingestion. The provider abstraction should feed that pipeline rather than replace it.
- `acorn_core::Repository` is used outside bucket transfer for source-code and artifact repositories. Expanding it with document databases or record files would increase exhaustive-match coupling in unrelated domains.

# Decisions

## Bucket source type

- Introduce a public `BucketSource` enum owned by `acorn-lib::io::config`, initially with only a repository-backed variant wrapping `acorn_core::Repository`.
- Rename the internal `Bucket` field from `code_repository` to `source`, but preserve the existing `repository` and `codeRepository` wire shapes for the repository variant. Existing JSON, JSONC, and YAML configurations must deserialize unchanged, and the current canonical serialized form must remain stable.
- Reject configurations that supply conflicting source fields instead of choosing one by precedence.
- Keep `acorn_core::Repository` unchanged. New bucket providers extend `BucketSource`, not the cross-domain repository enum.

The target configuration-facing shape is:

```rust
/// Configured origin of one bucket.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
#[serde(untagged)]
pub enum BucketSource {
    /// Existing local or remote repository source.
    Repository(Repository),
}

/// Configured collection of research activity files.
#[skip_serializing_none]
#[derive(Builder, Clone, Debug, Deserialize, Serialize)]
#[serde(try_from = "BucketInput")]
#[serde(rename_all = "camelCase")]
#[builder(start_fn = init)]
pub struct Bucket {
    /// Display name.
    pub name: Option<String>,
    /// Human-readable description.
    pub description: Option<String>,
    /// Provider used to enumerate and retrieve bucket items.
    #[serde(rename = "codeRepository", alias = "repository")]
    pub source: BucketSource,
}
```

`#[serde(untagged)]` keeps the wrapped `Repository` object at the same depth. The explicit field rename preserves the current canonical serialized key:

```json
{
  "name": "science",
  "codeRepository": {
    "provider": "git",
    "location": "file:./science"
  }
}
```

The documented input alias remains valid:

```json
{
  "name": "science",
  "repository": {
    "provider": "gitlab",
    "id": 42,
    "location": "https://gitlab.example/research/science"
  }
}
```

Supplying more than one of `source`, `repository`, and `codeRepository` is a duplicate source-field error. This prerequisite adds input shorthands but no new provider variant. After it lands, the Cosmos plan extends the same enum without changing `Bucket` or `Repository`:

```rust
pub enum BucketSource {
    Repository(Repository),
    Cosmos(CosmosBucketSource),
}
```

The Cosmos arm is illustrative follow-on code and is outside this plan's implementation boundary.

## Definition ergonomics and simplified input

Accept three equivalent levels of detail and normalize all of them immediately to `Bucket { source: BucketSource, .. }`:

1. A bare local path or repository URL for the common case.
2. A named object with a simple `source` string when labels or descriptions matter.
3. The existing `repository` or `codeRepository` object when provider-specific fields are required.

The shortest JSON form is:

```json
{
  "buckets": [
    "./science",
    "https://github.com/example/research-activities",
    "https://gitlab.example/research/science"
  ]
}
```

Names and descriptions can be added without spelling out the provider object:

```json
{
  "buckets": [
    {
      "name": "science",
      "description": "Reviewed science activities",
      "source": "https://gitlab.example/research/science"
    }
  ]
}
```

The same shorthand remains readable in YAML:

```yaml
buckets:
  - ./science
  - name: science
    source: https://gitlab.example/research/science
```

Use private input-only helpers so runtime code sees one normalized type and serialization still emits the existing detailed `codeRepository` object:

```rust
#[derive(Deserialize)]
#[serde(untagged)]
enum BucketInput {
    Simple(String),
    Detailed(BucketDetailsInput),
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct BucketDetailsInput {
    name: Option<String>,
    description: Option<String>,
    #[serde(rename = "codeRepository", alias = "repository", alias = "source")]
    source: BucketSourceInput,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum BucketSourceInput {
    Simple(String),
    Detailed(BucketSource),
}
```

`TryFrom<BucketInput> for Bucket` performs normalization. `ApplicationConfiguration.buckets` remains `Option<Vec<Bucket>>`; do not leak an input union into runtime callers. Bare strings set `name` and `description` to `None`; do not derive a persistent name from a path or URL. The named shorthand is available when a stable label is needed for progress and provenance.

Extract the current string classification from `Bucket::from(&str)` into one pure, fallible parser and use it for configuration and CLI input:

```rust
impl From<Repository> for BucketSource {
    fn from(repository: Repository) -> Self {
        Self::Repository(repository)
    }
}

impl TryFrom<&str> for BucketSource {
    type Error = Report;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        parse_repository_source(value).map(Self::Repository)
    }
}

impl TryFrom<&str> for Bucket {
    type Error = Report;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        BucketSource::try_from(value).map(|source| Self {
            name: None,
            description: None,
            source,
        })
    }
}
```

Preserve the current classification behavior for supported shorthand values: filesystem paths and `file:` URIs become local Git sources, `github.com` URLs become GitHub sources, and other valid remote repository URLs use the existing GitLab fallback. Relative paths retain the same project/current-directory resolution semantics as detailed repository locations; this plan does not silently rebase them to the configuration file's parent. Empty strings, malformed locations, embedded URL credentials, and unsupported schemes return configuration errors; parsing must never call `process::exit`. Provider-specific options such as GitLab numeric IDs, revisions, Cosmos coordinates, and future credentials require the detailed object form.

Keep the shorthand deliberately small. Do not add a name-to-source map form, shell expansion, environment interpolation, provider guessing from file contents, or a second parser for CLI arguments. Canonical serialization and configuration rewriting always use the detailed object form so shorthand is an authoring convenience rather than another persistent schema.

## Provider-neutral item contract

- Introduce a `BucketSourceItem` carrying an opaque provider key, a safe relative virtual path used by shared filtering and destination planning, and only the optional bounded metadata required by transfer and provenance.
- Define a crate-private adapter contract that lists source items deterministically and retrieves a selected item. Provider code owns authentication, pagination, revision resolution, and byte retrieval; it does not own filters, ignores, flattening, destination paths, clobbering, canonicalization, ingestion, progress summaries, or CLI rendering.
- Preserve streaming or bounded retrieval for existing remote repositories. The abstraction must not require every repository file to be buffered in memory.
- Prefer explicit `BucketSource` dispatch over a dynamic provider registry. The contract exists to isolate provider behavior, not to add runtime plugin discovery.

Use the existing `acorn_host::fs::SafePath` invariant for every provider-reported virtual path:

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BucketSourceItem {
    pub(crate) key: String,
    pub(crate) path: SafePath,
    pub(crate) revision: Option<String>,
    pub(crate) size: Option<u64>,
}
```

- `key` is an opaque provider locator used only to retrieve that listed item. It must not contain credentials and must not be treated as an output path.
- `path` is the normalized relative path used for filters, collision detection, destination planning, manifests, and user-facing progress.
- `revision` is an optional immutable provider version such as a repository commit. It supports stable retrieval and provenance without changing selection semantics.
- `size` is an optional declared byte count used for bounds and progress. Retrieval must still enforce actual response bounds when it is present or absent.

Construction must call `SafePath::new` and reject empty, absolute, traversal, and foreign-drive paths. Listing validation rejects duplicate normalized paths before filtering or writing. Keep fields crate-visible only if that is required by adjacent config modules; otherwise use a validating constructor plus read-only accessors.

The adapter surface remains crate-private and statically dispatched:

```rust
pub(crate) trait BucketSourceAdapter {
    fn is_local(&self) -> bool;
    fn manifest_identity(&self) -> String;
    async fn items(&self) -> ApiResult<Vec<BucketSourceItem>>;
    async fn retrieve(&self, item: &BucketSourceItem, destination: &Path) -> ApiResult<()>;
}

pub(crate) struct RepositoryBucketSource<'a> {
    pub(crate) repository: &'a Repository,
}
```

The shared transfer layer, not the adapter, selects and prepares `destination`. `retrieve` receives that already validated target so the repository adapter can preserve streaming downloads and GitLab fallback behavior without gaining control of filtering, flattening, clobbering, or manifest construction. Implementations use static enum matching; the plan does not create `Box<dyn BucketSourceAdapter>` or a registry.

The planned ownership of the data structures is:

| Structure | Owner | Responsibility |
|---|---|---|
| `BucketSource` | `config/bucket_source.rs` | Serialized provider selection and explicit dispatch. |
| `BucketInput` / `BucketSourceInput` | `config/bucket_source.rs` | Input-only shorthand normalization; never exposed to runtime consumers or serialization. |
| `RepositoryBucketSource` | `config/bucket_source.rs` | Existing repository enumeration, revision resolution, authentication, and retrieval. |
| `BucketSourceItem` | `config/bucket_source.rs` | Provider key plus safe, deterministic transfer metadata. |
| `TransferItem` | `config/transfer.rs` | Selected source path and resolved destination after filtering and flattening. |
| `TransferManifest` | `config/transfer.rs` | Successfully written relative paths, existing source identity, canonicalization, provenance, and ingestion. |

## Repository adapter and transfer ownership

- Move local enumeration, remote repository listing, branch selection, raw-file retrieval, authentication, and GitLab fallback behind the repository-backed adapter while preserving their current results and failure policy.
- Apply shared filters only after deterministic item discovery and before provider retrieval. Convert selected items to the existing `TransferItem` and `TransferManifest` flow so destination safety, collision handling, concurrency, canonicalization, provenance, and ingestion keep one owner.
- Preserve the existing local-versus-remote and import-versus-download policy: download rejects local sources, offline mode rejects remote sources before network or credential access, and import continues to accept local repositories and verified archive payloads.
- Preserve the current transfer-manifest and bucket-provenance wire contracts in this refactor. Provider-specific provenance extensions belong to the concrete provider plan that needs them.

The orchestration remains a single directional pipeline:

```text
BucketSourceAdapter::items
    -> validate and order BucketSourceItem values
    -> apply FilterSet
    -> build TransferItem values and resolve flattening collisions
    -> transfer_bucket_files
        -> prepare each safe destination
        -> BucketSourceAdapter::retrieve
    -> TransferManifest::canonicalize
    -> TransferManifest::ingest
```

For repository-backed buckets, `manifest_identity()` returns the same repository location string currently passed to `transfer_bucket_files`. The existing `TransferManifest::repository` and `Provenance::Bucket::repository` fields therefore remain unchanged in this prerequisite even though the runtime selector is now named `BucketSource`.

## Scope boundary

- Do not add Cosmos DB, JSON Lines, object storage, SQL, or any other new source variant.
- Do not add uploads, writes, synchronization, change feeds, a provider registry, new CLI arguments, or a new top-level command.
- Do not infer Cosmos or another future provider from a shorthand string; non-repository providers require an explicit detailed source object.
- Do not generalize the existing one-shot `io::source::Source`; transient file/URL reads and configured bucket providers remain separate concepts.
- Do not change which repository variants can currently act as buckets or broaden their authentication behavior.

# Recommendation

Freeze configuration and transfer behavior with characterization tests first. Add the source type and one fallible string parser, normalize shorthand during deserialization, then adapt repositories behind the narrow list/retrieve contract. Switch `Bucket` orchestration only after the adapter produces equivalent items and manifests. Keep the provider contract crate-private until a second implementation proves that a larger public interface is necessary.

# Implementation sequence

1. Add serialization and production-path characterization tests for repository-backed buckets, including both accepted field names, canonical serialization, local and remote policy, item ordering, manifests, provenance, filters, flattening, clobbering, and ingestion.
2. Add `BucketSource`, the private input helpers, and one fallible string parser. Cover bare strings, named `source` strings, detailed source objects, equivalent normalization, conflicts, and canonical serialization.
3. Replace process-exiting string conversion in configuration, import, and download paths with the shared fallible parser and actionable errors.
4. Add `BucketSourceItem` and the crate-private adapter contract in `bucket_source.rs`, with repository-only construction and no new dependencies.
5. Move repository discovery and selected-item retrieval behind `RepositoryBucketSource`, retaining current streaming, authentication, branch, fallback, and error behavior.
6. Route `Bucket` import and download orchestration through `BucketSource`, then remove direct repository dispatch from the shared transfer path.
7. Document the bare, named shorthand, and detailed forms without advertising a provider that does not yet exist.

# Validation

- Deserialize existing JSON, JSONC, and YAML fixtures using `repository` and `codeRepository`; assert equivalent `BucketSource::Repository` values and unchanged canonical serialization.
- Deserialize bare string entries and named `source` strings in JSON, JSONC, and YAML; assert they normalize identically to their detailed repository objects and serialize canonically as `codeRepository`.
- Reject empty or malformed shorthand, embedded credentials, unsupported schemes, missing sources, unknown fields where currently rejected, and conflicting `source`/`repository`/`codeRepository` fields with actionable configuration paths.
- Exercise the same shorthand through CLI import and download parsing and prove failures return errors rather than terminating inside library code.
- Construct `BucketSourceItem` values from valid cross-platform relative paths and reject empty, absolute, parent-traversal, foreign-drive, and duplicate paths before retrieval.
- Compare pre-refactor and post-refactor item order, selected paths, transfer manifests, provenance, counts, output trees, and local-database candidates for local, GitHub, and GitLab fixtures.
- Exercise include and ignore filters, flattening collisions, clobbering, canonicalization, strict ingestion, `--no-local-database`, quiet output, archive-derived local buckets, and partial transfer failures through production command paths.
- Verify offline remote import and download fail before authentication or network access, while local import remains available and local download remains rejected.
- Run focused configuration, import, and download tests, then `make lint`, `make test`, `make check-portable`, and `make check-wasm`.

# Acceptance criteria

- `Bucket` owns a typed `BucketSource` rather than an `acorn_core::Repository`, and `acorn_core::Repository` gains no bucket-only variants.
- Existing `repository` and `codeRepository` configurations and canonical serialization remain compatible.
- Users can define a common bucket as a bare path/URL or a named object with a `source` string, while provider-specific configuration retains the detailed object form.
- Configuration and CLI shorthand share one fallible parser, and invalid input never exits the process from library code.
- Repository-backed sources implement the documented `BucketSourceAdapter` contract with `SafePath`-validated `BucketSourceItem` values, and the Cosmos plan can extend it without duplicating transfer orchestration.
- Shared filtering, destination safety, collisions, clobbering, concurrency, manifests, canonicalization, provenance, ingestion, and CLI summaries remain outside provider adapters and have one implementation.
- Existing local, GitHub, and GitLab bucket behavior, including streaming, authentication, fallback, offline rejection, and observable errors, has regression coverage and remains unchanged.
- No new provider, CLI surface, upload path, runtime registry, or cross-domain `Repository` variant is introduced.
