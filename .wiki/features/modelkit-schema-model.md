---
okf_version: "0.1"
type: Plan
title: Portable ModelKit schema model
description: Model the KitOps ModelKit v0.1 and Kitfile 1.0.0 contracts in acorn-schema, including distinct authoring YAML and resolved OCI-config JSON representations.
difficulty: high
impact: high
package:
  - acorn-schema
  - acorn-lib
tags:
  - modelkit
  - kitops
  - schema
  - oci
  - yaml
  - json
status: planned
---

# Project code boundaries

- **Update:** `crates/acorn-schema/src/schema/mod.rs`, `crates/acorn-lib/src/io/oci.rs`, `crates/acorn-lib/src/util/constants/app.rs`, and dedicated tests for those modules.
- **Create:** `crates/acorn-schema/src/schema/modelkit/mod.rs`, focused child modules only if the implementation no longer fits one reviewable file, and producer-derived fixtures under `tests/fixtures/modelkit/`.
- **Delete:** The private `ModelKitConfig` and `ModelKitModel` DTOs in `crates/acorn-lib/src/io/oci.rs` after that consumer migrates to the portable model. No generic OCI manifest, transport, or archive types move as part of this plan.

# Dependencies

- Pin the compatibility baseline to KitOps v1.15.0, tag commit `6b8162ae5da4d46f1d2af2beb43e7fb077f052f4`, and record the exact upstream files used by each fixture. Re-evaluate the pin immediately before implementation.
- Land the portable schema and codecs before the ModelKit slice of [ModelKit and ModelPack OCI interoperability](oci-model-artifact-downloads.md). That plan remains responsible for OCI classification, blob verification, layer selection, extraction, and GGUF policy.
- Reuse `acorn-schema`'s ordered `ValidationReport`, portable allocation types, `serde`, `schemars`, `serde_norway`, and `serde_json`. Do not add a runtime KitOps dependency or invoke the `kit` executable from the schema crate.
- This plan occupies the **Schema features** lane in the [parallel execution guide](../parallel-execution.md). `schema/mod.rs` is a module-index lock; `io/oci.rs` and ModelKit constants are handed to the Provider-leaves integration owner for the final consumer migration.

# Portable ModelKit schema model

## Goal

Provide a public, portable Rust model for the current KitOps ModelKit contract so ACORN can parse, validate, inspect, generate JSON Schema for, and round-trip Kitfiles without reducing them to `model.path`. The model must cover package metadata, model and model parts, datasets, code, documentation, prompts, MCP Bundles, arbitrary JSON-compatible parameters, and resolved layer identity.

ModelKit has two related wire representations that must remain explicit:

1. An authoring `Kitfile` is YAML with `manifestVersion` and at least one content section. Paths and user metadata are authored here; derived layer digests are not.
2. The OCI config blob uses media type `application/vnd.kitops.modelkit.config.v1+json`. It serializes the Kitfile-shaped data as JSON and enriches content entries with `digest` and `diffId` values that bind them to OCI layers.

The OCI image manifest is already parsed and validated inside `acorn-lib::io::oci`. This plan does not publish a second generic OCI descriptor hierarchy from `acorn-schema`; it models the ModelKit-specific config contract and media-type vocabulary, then lets the transport layer compose them.

## Current evidence and gaps

| Concern | Current ACORN behavior | Target behavior |
|---|---|---|
| Public schema | No ModelKit or Kitfile type exists in `acorn-schema`. | Public `modelkit` types expose the supported 1.0.0 contract with `JsonSchema`, serde, equality, and validation support. |
| OCI config parsing | `io/oci.rs` privately decodes only `model.path`. | Decode and validate the complete resolved config, while the model downloader reads only the normalized model fields it needs. |
| YAML versus JSON | No authoring codec exists; derived layer metadata is invisible. | Authoring YAML never emits or accepts producer-derived layer identity; resolved JSON preserves and validates it. |
| Content coverage | Model-free kits are rejected by the model downloader before they can be represented. | The schema accepts valid model-free kits containing datasets, code, docs, prompts, or MCP servers; the downloader applies its separate “must contain a model” policy. |
| Validation | Only the selected model path receives OCI-local safety checks. | Pure validation covers version policy, content presence, path/reference kinds, duplicate and overlapping paths, remote dataset integrity, model-part types, MCP Bundle constraints, and unique MCP server names. |
| Compatibility | No pinned producer fixture or drift boundary exists for ModelKit config fields. | Golden YAML/JSON fixtures and differential tests record behavior against one pinned KitOps release and identify documentation/producer differences. |

## Recommended model

### 1. Define a portable domain vocabulary

Add `schema::modelkit` with a small public type set:

- `Kitfile`: `manifest_version`, optional `package` and `model`, and collections for `datasets`, `code`, `docs`, `prompts`, and `mcp_servers`.
- `Package`: name, version, description, license, and authors.
- `Model`: name, path, license, framework, format, version, description, parts, JSON-compatible parameters, and optional resolved layer identity.
- `ModelPart`: name, path, license, kind, and optional resolved layer identity.
- `Dataset`: name, local path, `remote_path`, `remote_hash`, description, license, JSON-compatible parameters, and optional resolved layer identity.
- `Code`, `Document`, `Prompt`, and `McpServer`: only their specified fields plus optional resolved layer identity. Do not introduce a public inheritance-style “layer base” abstraction merely to remove repeated `path` and `description` fields.
- `LayerIdentity`: compressed digest and uncompressed `diff_id`, both optional in the domain so one value can represent authoring and resolved states.
- `Parameters`: a transparent JSON-compatible value or map that rejects YAML-only tags/types and preserves nested null, boolean, number, string, array, and object values.

Use Rust names while preserving upstream keys with serde renames (`manifestVersion`, `mcpServers`, `remotePath`, `remoteHash`, and `diffId`). Preserve ordered deterministic diagnostics and serialization; do not normalize user strings or paths during deserialization.

Exit condition: the complete pinned KitOps struct shape has a documented Rust counterpart, generated JSON Schema is stable, and no host/filesystem/process dependency is introduced into `acorn-schema`.

### 2. Separate authoring and resolved codecs

Implement explicit codecs or private wire DTOs around the domain model:

- `from_yaml` parses authoring Kitfiles with unknown-field rejection and aggregated deserialization errors. It rejects `digest` and `diffId` because KitOps marks those fields JSON-only and derives them while packing.
- `to_yaml` omits all resolved layer identity even when the domain value came from an OCI config blob.
- `from_resolved_json` accepts the full config blob, including `digest` and `diffId`, and reports malformed identity separately from semantic Kitfile validation.
- `to_resolved_json` preserves resolved identity and uses deterministic object ordering for parameter maps where the underlying serializer permits it.
- Both decoders reject trailing documents/data and distinguish syntax/deserialization failures from semantic validation reports.

Do not rely on one `Serialize` implementation to encode both formats: upstream's Go tags intentionally serialize layer identity to JSON and suppress it from YAML. Keep the format distinction visible in API names and tests.

Exit condition: a resolved JSON fixture can round-trip without losing layer identity, while its YAML projection contains only authoring fields.

### 3. Reproduce the normative validation contract deliberately

Implement pure whole-value validation that aggregates independent issues. Match the pinned producer where it is more specific than the prose docs, and record discrepancies rather than silently inventing constraints:

1. Recognize `manifestVersion: 1.0.0`. Preserve an unrecognized version long enough to issue an unsupported-version diagnostic; do not deserialize the version into a closed enum that destroys forward-compatible evidence. Decide before implementation whether ACORN's public validation treats KitOps' warning as a warning result or an error at strict entry points.
2. Require at least one of model, datasets, code, docs, prompts, or MCP servers for spec-valid authoring, even though the pinned producer's `Validate` method does not currently enforce the prose requirement. Cover this documented divergence with a fixture.
3. Reject absolute local paths, invalid URLs, duplicate normalized content paths, and local content nested inside a referenced remote-dataset destination. Define normalization lexically and platform-independently; validation must not query the filesystem or follow symlinks.
4. Permit model paths that are local or ModelKit references. Permit only local paths for model parts, code, docs, prompts, and MCP servers. Permit dataset `remotePath` values only for supported S3 or ModelKit references; require `remoteHash` for S3, ignore-with-warning or reject it for ModelKit references according to the selected strictness mode, and reject `remoteHash` without `remotePath`.
5. Limit a non-empty model-part `type` to the pinned producer's identifier grammar and maximum length of 64 bytes. Model the field as `kind` in Rust with `#[serde(rename = "type")]`.
6. Require each MCP server name, enforce uniqueness, and require a local `.mcpb` file path. Do not inspect the ZIP or duplicate its internal `manifest.json` in this schema slice.
7. Validate layer identities as OCI digests when present and state whether an authoring value carrying them is rejected by the codec before semantic validation.

Keep validation separate from download policy: GGUF extensions, supported compression, layer/media-type matching, archive safety, remote access, and model selection remain `acorn-lib` concerns.

Exit condition: one invalid fixture can report multiple stable, path-addressed issues, and tests document every intentional difference from KitOps v1.15.0.

### 4. Migrate the existing OCI consumer

After the portable contract is green, replace `ModelKitConfig` and `ModelKitModel` in `io/oci.rs` with `from_resolved_json` and the public model:

1. Verify the config blob's descriptor size and digest before decoding, as owned by the OCI interoperability plan.
2. Parse and validate the complete resolved config, then apply the command-specific requirement that `model` exists.
3. Select model/model-part paths and layer identities through pure adapter functions. Do not make `acorn-schema` depend on `OciLayer`, ORAS, filesystems, or GGUF.
4. Retain unrelated valid content in the parsed config for inspection and diagnostics, but do not materialize it implicitly during `download model`.
5. Map unsupported schema versions, invalid Kitfiles, missing model content, and layer-association failures to distinct diagnostics.

Exit condition: the existing ModelKit download tests use the public schema, model-free kits parse successfully before command-policy rejection, and the private two-field DTOs are deleted.

## Validation

### Portable schema and codec tests

- Minimal fixtures for each permitted content section and one full Kitfile containing every field from the pinned v1.15.0 types.
- YAML and resolved-JSON golden tests, including camel-case keys, omitted optional values, arbitrary nested parameters, and `digest`/`diffId` asymmetry.
- Unknown fields, duplicate keys, trailing YAML documents, trailing JSON data, YAML-only parameter values, malformed numbers, and aggregate deserialization errors.
- Empty content, recognized/unrecognized versions, absolute and traversal-like paths, Windows and POSIX path spellings, URLs, ModelKit references, S3 datasets and hashes, duplicate/overlapping paths, model-part types, MCP suffixes, and duplicate MCP names.
- `schemars` snapshot tests plus `cargo check -p acorn-schema --no-default-features --features alloc` and the existing WASM package check.

### Differential and consumer tests

- Generate fixtures with the pinned `kit` binary, compare ACORN acceptance and diagnostics with `kit inspect` or `kit pack`, and record intentional differences such as the documented non-empty-content rule.
- Decode a config blob taken from a producer-generated ModelKit and prove that every resolved layer identity survives JSON round-trip and disappears from YAML projection.
- Run focused `acorn-schema` ModelKit tests and `acorn-lib` OCI tests, then `make lint`, `make test`, `make wasm`, and `make audit`.

## Risks and mitigations

- **The prose documentation and producer validation differ.** Treat the pinned producer types plus published specification as separate evidence, encode each discrepancy in tests, and do not call behavioral parity “spec compliance” without qualification.
- **ModelKit evolves independently of ModelPack.** Keep the namespace, fixtures, media types, and version policy separate; a KitOps-produced ModelPack continues through the ModelPack parser.
- **One serde representation would lose data or leak derived fields into YAML.** Use explicit authoring/resolved codec boundaries and cross-format tests.
- **Host-dependent path cleaning can make validation platform-specific.** Use a portable lexical path policy in the schema and leave filesystem containment to materialization.
- **Arbitrary parameters can undermine portability.** Accept only the JSON data model and test normalization of numeric and mapping forms documented by KitOps.
- **Public types can freeze producer accidents.** Publish only the documented domain fields and resolved layer identity; keep codec DTOs and compatibility shims private.

## Acceptance criteria

- `acorn-schema` publicly models every Kitfile field present in the pinned KitOps v1.15.0 `KitFile`, `Package`, `Model`, `ModelPart`, `DataSet`, `Code`, `Docs`, `Prompt`, `MCPServer`, and `LayerInfo` definitions.
- The model is portable under the crate's alloc-only and WASM checks and introduces no host, network, filesystem, or process dependency.
- Authoring YAML rejects unknown/derived fields and never emits `digest` or `diffId`; resolved JSON parses and round-trips both fields.
- Validation aggregates deterministic issues for version/content, path/reference, duplicate/overlap, remote dataset, model-part, MCP server, and digest rules.
- Model-free Kitfiles are valid schema values when they contain another supported component, while `download model` reports a separate missing-model policy error.
- The OCI consumer uses the public resolved-config decoder and deletes its private `ModelKitConfig`/`ModelKitModel` subset types.
- Producer-derived fixtures are pinned to an immutable KitOps source revision, and documented differences between the specification and v1.15.0 behavior are covered by tests.
- Focused schema/OCI tests and repository lint, test, WASM, and audit checks have accepted results.

## Evidence

- KitOps defines ModelKit as an OCI-compatible bundle of model, dataset, code, prompt, MCP server, and documentation artifacts: <https://kitops.org/docs/modelkit/intro/>.
- The ModelKit v0.1 specification defines the OCI-backed artifact roles and the `application/vnd.kitops.modelkit.config.v1+json` Kitfile config media type: <https://kitops.org/docs/modelkit/spec/>.
- The Kitfile overview states that `manifestVersion` and at least one content section are required, paths are relative, and a kit may contain no model: <https://kitops.org/docs/kitfile/kf-overview/>.
- The Kitfile format documents package, model, model parts, datasets, code, docs, prompts, MCP Bundles, and JSON-compatible parameter values: <https://kitops.org/docs/kitfile/format/>.
- The pinned producer types and validation behavior are defined by KitOps v1.15.0 at commit [`6b8162ae5da4d46f1d2af2beb43e7fb077f052f4`](https://github.com/kitops-ml/kitops/blob/6b8162ae5da4d46f1d2af2beb43e7fb077f052f4/pkg/artifact/kitfile.go).
- KitOps' TypeScript types independently expose the public Kitfile and OCI manifest shapes, but currently omit newer MCP-server fields; this is evidence for pinning the Go producer source rather than merging unversioned documentation ad hoc: <https://kitops.org/docs/kitops-ts/types/>.
- ACORN currently defines only `ModelKitConfig { model }` and `ModelKitModel { path }` in `crates/acorn-lib/src/io/oci.rs`; the remainder of a valid resolved config is discarded.
