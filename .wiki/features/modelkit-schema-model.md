---
okf_version: "0.1"
type: Plan
title: Portable ModelKit and Kitfile schema model
description: Model the KitOps ModelKit v0.1 and Kitfile 1.0.0 contracts (pkg/artifact/kitfile.md + pkg/artifact/kitfile.go) in acorn-schema, including distinct authoring YAML and resolved OCI-config JSON representations matching KitOps behavior.
difficulty: high
impact: high
package:
  - acorn-schema
  - acorn-lib
tags:
  - modelkit
  - kitfile
  - kitops
  - schema
  - oci
  - yaml
  - json
status: in-progress
---

# Project code boundaries

- **Update:** `crates/acorn-schema/src/schema/mod.rs`, `crates/acorn-lib/src/io/oci.rs`, `crates/acorn-lib/src/util/constants/oci.rs`, and dedicated tests for those modules.
- **Create:** `crates/acorn-schema/src/schema/modelkit/mod.rs`, the adjacent `kitfile.rs` implementation module, and producer-derived fixtures under `tests/fixtures/modelkit/`.
- **Delete:** The private `ModelKitConfig` and `ModelKitModel` DTOs in `crates/acorn-lib/src/io/oci.rs` after that consumer migrates to the portable model. No generic OCI manifest, transport, or archive types move as part of this plan.

# Dependencies

- Pin the compatibility baseline to KitOps v1.15.0, tag commit `6b8162ae5da4d46f1d2af2beb43e7fb077f052f4`, for both `pkg/artifact/kitfile.go` (canonical types, `Validate`, `GetPathType`/`IsModelKitReference`, `LayerInfo` `yaml:"-"`) and `pkg/artifact/kitfile.md` (130-line format reference). `kitfile.md` is a strict subset — it omits `package.license`, `model.format`, `modelpart.license`, `dataset.remotePath`/`remoteHash`/`parameters`, `prompt.name`, and `LayerInfo` detail. Treat the Go source as canonical where they diverge and record the divergence in tests. Record the exact upstream files+commit used by each fixture. Re-evaluate the pin immediately before implementation.
- Land the portable schema and codecs before the ModelKit slice of [ModelKit and ModelPack OCI interoperability](oci-model-artifact-downloads.md). That plan remains responsible for OCI classification, blob verification, layer selection, extraction, and GGUF policy.
- Reuse `acorn-schema`'s ordered `ValidationReport`, portable allocation types, `serde`, `schemars`, `serde_norway`, and `serde_json`. Do not add a runtime KitOps dependency or invoke the `kit` executable from the schema crate.
- This plan occupies the **Schema features** lane in the [parallel execution guide](../parallel-execution.md). `schema/mod.rs` is a module-index lock; `io/oci.rs` and ModelKit constants are handed to the Provider-leaves integration owner for the final consumer migration.

# Portable ModelKit and Kitfile schema model

## Implementation progress (2026-09-05)

- `acorn-schema::schema::modelkit` now publicly models every field in the pinned KitOps v1.15.0 `KitFile`, `Package`, `Model`, `ModelPart`, `DataSet`, `Code`, `Docs`, `Prompt`, `MCPServer`, and `LayerInfo` types.
- Strict authoring-YAML and resolved-config JSON entry points reject unknown and duplicate keys, trailing input, YAML-only tagged values, and derived identity in authoring YAML. Resolved JSON preserves `digest` and `diffId`; YAML projection removes them.
- Portable validation covers content presence, version warnings, local/ModelKit/S3 path policy, normalized duplicate and remote-dataset collisions, model-part types, MCP Bundle rules, the shared lowercase SHA-256 digest rule, and independent all-or-none digest/diff-ID completeness. Intentional differences are the documented non-empty-content requirement, portable path cleaning, prompt duplicate-path participation, and MCP identity completeness.
- Package/model versions and licenses now compose shared scalar validators through `optional(rule)`, remote dataset paths use a dedicated field validator, model format is typed with a forward-compatible fallback, and portable path/reference classification and normalization live on `acorn_core::Location`.
- `Kitfile` I/O now implements the shared `InputOutput` contract for authoring YAML and resolved JSON through the host adapter.
- `Kitfile` and `ModelKit` are selectable analyzer standards and run through the shared `Check` schema-validation flow, including automatic standard detection.
- The OCI consumer now uses the public resolved decoder, distinguishes valid model-free packages at the command-policy boundary, and associates the primary GGUF plus GGUF model parts through resolved layer digests. The private `ModelKitConfig`/`ModelKitModel` DTOs were deleted.
- Verbatim fixtures are pinned to KitOps v1.15.0 commit `6b8162ae5da4d46f1d2af2beb43e7fb077f052f4`; complete authoring and resolved fixtures exercise every modeled field. Focused schema, codec, analyzer, alloc-only, and OCI consumer tests pass, as does the pinned upstream `go test ./pkg/artifact` suite.
- Repository validation passed `make lint` and `make test` (1,482 passed and 14 skipped); earlier validation also passed `make check-wasm` and `make book`. The repository-wide `make audit` gate remains unaccepted because its existing cargo-udeps configuration reports `keyring`, `arboard`, and `itertools`; this implementation added no dependency.

## Goal

Provide a public, portable Rust model that **matches KitOps** for the current KitOps ModelKit contract so ACORN can parse, validate, inspect, generate JSON Schema for, and round-trip `Kitfile`s without reducing them to `model.path`. The model must cover package metadata, model and model parts, datasets, code, documentation, prompts, MCP Bundles, arbitrary JSON-compatible parameters, and resolved layer identity — faithful to `pkg/artifact/kitfile.go` and `pkg/artifact/kitfile.md`.

ModelKit has two related wire representations that must remain explicit (KitOps `LayerInfo` Go tags: `json:"digest,omitempty" yaml:"-"`, `json:"diffId,omitempty" yaml:"-"`):

1. An authoring `Kitfile` is YAML with `manifestVersion` and at least one content section. Paths and user metadata are authored here; derived layer digests are not. This is the `kitfile.md` format reference (`manifestVersion`, `package`, `code`, `datasets`, `docs`, `prompts`, `mcpServers`, `model` with `parts`/`parameters`). The Go source adds fields `kitfile.md` omits (`package.license`, `model.format`, `modelpart.license`, `dataset.remotePath`/`remoteHash`/`parameters`, `prompt.name`, `authors` flow style) — these remain in scope.
2. The OCI config blob uses media type `application/vnd.kitops.modelkit.config.v1+json`. It serializes the Kitfile-shaped data as JSON and enriches content entries with `digest` and `diffId` values that bind them to OCI layers (including `application/vnd.kitops.modelkit.mcpb.v1.raw` for verbatim `.mcpb` bundles noted in `kitfile.md`).

The OCI image manifest is already parsed and validated inside `acorn-lib::io::oci`. This plan does not publish a second generic OCI descriptor hierarchy from `acorn-schema`; it models the ModelKit-specific config contract and media-type vocabulary, then lets the transport layer compose them.

## Current evidence and gaps

| Concern | Current ACORN behavior | Target behavior |
|---|---|---|
| Public schema | Public `modelkit` types expose the pinned 1.0.0 contract with `JsonSchema`, serde, equality, and validation support. | Review additions deliberately when the pinned producer baseline changes. |
| OCI config parsing | `io/oci.rs` uses the public resolved decoder and reads normalized model/model-part layer identities. | Extend materialization formats separately from schema compatibility when required. |
| YAML versus JSON | Authoring YAML rejects and omits derived identity; resolved JSON preserves and validates it. | Retain the explicit codec boundary. |
| Content coverage | Model-free kits containing another content section are valid schema values; the model downloader applies its separate missing-model policy. | Preserve package validity independently of command policy. |
| Validation | Pure validation aggregates deterministic path/reference, collision, remote dataset, model-part, MCP, and layer-identity issues. | Expand pinned differential fixtures when KitOps behavior changes. |
| Compatibility | Verbatim producer fixtures pin YAML/JSON normalization and MCP validation to KitOps v1.15.0. | Add a reproducible pinned-binary differential job before retiring the plan. |

## Recommended model

### 1. Define a portable domain vocabulary matching `kitfile.go`/`kitfile.md`

Add `schema::modelkit` with a small public type set that is 1:1 with the pinned Go structs (Rust names preserve upstream keys via serde renames):

- `Kitfile`: `manifest_version: String` (`manifestVersion`), semantically optional `package`, optional `model: Option<Model>`, and collections for `datasets` (`DataSets` in Go, `json:"datasets"`), `code`, `docs`, `prompts`, and `mcp_servers` (`mcpServers`). Mirrors `type KitFile struct` in `kitfile.go:46-55` and sections in `kitfile.md:18-99`. The Go producer stores `Package` by value, so missing input becomes an empty package and its JSON encoder may emit `{}` despite `omitempty`; ACORN should accept absent and empty forms without making that encoder quirk normative.
- `Package`: name, version, description, license (`kitfile.md` omits but Go `kitfile.go:58-64` has it), and `authors: Vec<String>` (`yaml:",flow"` — no behavioral effect but fixtures must tolerate flow style).
- `Model`: name, path, license, framework, format (`kitfile.md` omits but Go `kitfile.go:71-81` has it), version, description, `parts: Vec<ModelPart>`, JSON-compatible `parameters` (any, see below), and inline `LayerInfo`. Prompt/Model `name` fields exist in Go even when `kitfile.md` examples omit them.
- `ModelPart`: name, path, license, `kind` (`#[serde(rename = "type")]`), and inline `LayerInfo`.
- `Dataset`: name, local path, `remote_path`, `remote_hash`, description, license, JSON-compatible `parameters`, and inline `LayerInfo`. `kitfile.md` datasets section omits remote fields — they remain required for KitOps parity.
- `Code` (`kitfile.go:84-89`), `Docs` (`kitfile.go:67-69`), `Prompt` (`kitfile.go:100-104`), `McpServer` (`kitfile.go:112-118`): only their specified fields (`path`, `description`, plus `name` where Go defines it) plus inline `LayerInfo`. Do not introduce a public inheritance-style “layer base” abstraction merely to remove repeated `path` and `description` fields.
- `LayerInfo`: `digest: Option<String>` (`json:"digest,omitempty" yaml:"-"`), `diff_id: Option<String>` (`json:"diffId,omitempty" yaml:"-"`), both optional so one domain value can represent authoring and resolved states. Model digest completeness and diff-ID completeness as two independent results, matching `KitfileHasLayerInfo`; do not require both categories merely because one is complete.
- `Parameters`: transparent JSON-compatible value (`serde_json::Value` or equivalent) that rejects YAML-only tags/types. KitOps caveats (`kitfile.md:62-63` + `kitfile.go:76-79` comments): only JSON subset, strings sans flow params, numbers decimal (`0xFF -> 255`, `1.2e+3 -> 1200`), maps sorted alphabetically — cover with `testdata/parameters` fixtures.

Use Rust names while preserving upstream keys with serde renames (`manifestVersion`, `mcpServers`, `remotePath`, `remoteHash`, and `diffId`). Preserve ordered deterministic diagnostics and serialization; do not normalize user strings or paths during deserialization. Add the KitOps MCP bundle media type `application/vnd.kitops.modelkit.mcpb.v1.raw` (from `kitfile.md:48`) to `crates/acorn-lib/src/util/constants/app.rs` vocabulary if absent.

Exit condition: every field in pinned `KitFile`/`Package`/`Model`/`ModelPart`/`DataSet`/`Code`/`Docs`/`Prompt`/`MCPServer`/`LayerInfo` has a documented Rust counterpart, `kitfile.md` gaps are explicitly noted, generated JSON Schema is stable, and no host/filesystem/process dependency is introduced into `acorn-schema`.

### 2. Separate authoring and resolved codecs — match `kitfile.go` tags exactly

Implement explicit codecs or private wire DTOs around the domain model to reproduce Go `yaml:"-"` vs `json:",inline"`:

- `from_yaml` parses authoring Kitfiles via `serde_norway` (or current ACORN YAML) with `deny_unknown_fields` and `KnownFields(true)` behavior (`LoadModel:168-172`), aggregated `eserde`-style deserialization errors, single-document rejection, and **rejection of `digest`/`diffId`**. KitOps marks those `yaml:"-"` (`LayerInfo:128-131`); they are derived while packing and must not appear in authored YAML. Test with `pkg/artifact/testdata/validation` fixtures.
- `to_yaml` via `serde_norway` omits all resolved layer identity even when the domain value came from an OCI config blob (`MarshalToYAML:185-193` uses `yaml.Encoder` which respects `yaml:"-"`).
- `from_resolved_json` via `serde_json` (`MarshalToJSON:176-183`) accepts the full config blob, including `digest` and `diffId`, and reports malformed identity (non-OCI digest) separately from semantic Kitfile validation.
- `to_resolved_json` preserves resolved identity and uses deterministic object ordering for parameter maps where the underlying serializer permits it (KitOps sorts maps alphabetically for parameters).
- Both decoders reject trailing documents/data and distinguish syntax/deserialization failures from semantic validation reports.

Do not rely on one `Serialize` implementation to encode both formats: upstream's Go tags intentionally serialize layer identity to JSON and suppress it from YAML (`kitfile.go:67-104` inline `*LayerInfo`). Keep the format distinction visible in API names and tests. Reproduce KitOps `parameters` normalization (decimal numbers, sorted keys) as noted in `kitfile.md:62-63`.

Exit condition: a resolved JSON fixture (e.g., `testdata/parameters/test-*`) can round-trip without losing layer identity or parameter fidelity, while its YAML projection contains only authoring fields.

### 3. Reproduce the normative validation contract deliberately — match `kitfile.go:168-285` + `util.go`

Implement pure whole-value validation that aggregates independent issues in deterministic sorted order (`slices.Sort(errs)` and `strings.Join` in `kitfile.go:276-280`). Match the pinned producer where it is more specific than `kitfile.md`, and record discrepancies rather than silently inventing constraints. All path handling must be lexical and platform-independent (`filepath.Clean`, `path.IsAbs`/`filepath.IsAbs`, `filepath.Rel` in `kitfile.go:206-287`) — validation must not query the filesystem or follow symlinks:

1. Recognize `manifestVersion: 1.0.0` with KitOps warning path (`kitfile.go:180-181`): `Validate` appends warning `"Unrecognized manifestVersion %s: treating Kitfile as 1.0.0"` and continues. Preserve the version string long enough to issue the diagnostic; do not deserialize into a closed enum that destroys forward-compatible evidence. Decide before implementation whether ACORN's public validation surfaces this as a `ValidationReport` warning (KitOps prints via `output.Logln`) or as a hard error at strict entry points; document the choice.
2. Require at least one of model, datasets, code, docs, prompts, or MCP servers for spec-valid authoring per `kitfile.md:11` (“at least one content section”), even though `kitfile.go:Validate` does not currently enforce the prose requirement. Cover this documented divergence with a fixture (do not silently add enforcement without a test noting the KitOps mismatch).
3. Path-kind dispatch via `GetPathType`/`IsModelKitReference` (`kitfile.go:215-246`, `util.go:24-162`): `IsModelKitReference` returns true only when the string contains `:` or `@` and `ParseReference` succeeds (registry contains `.` or `:`, else defaults to `localhost`, repo lowercase `startEndAlphanumeric`, tag/digest validated). `GetPathType` classifies: `ModelReferencePathType` (reference-like), `S3PathType` (`s3://` scheme), `LocalPathType` (no scheme/host or parse error), `UnknownPathType` (`http`/`https` → error `"HTTP urls are not supported"`). Reproduce S3/ModelKit-refs cases from `testdata/validation` (`s3-path-for-*`, `modelkit-ref-for-*`).
4. Reject absolute local paths (`path.IsAbs`/`filepath.IsAbs` per `kitfile.go:268`), duplicate normalized content paths (`paths map[string][]string` with `filepath.Clean`, `"." ` for `""`, error `"%s and %s use the same path %s"`), and local content nested inside a referenced remote-dataset destination (`filepath.Rel` check with exclusion of the dataset's own path, `testdata/validation/remote-dataset-collides-*` + `sibling-paths-allowed`). Covers `duplicate-path-in-kitfile_*.yaml` fixtures.
5. Permit model paths that are local or ModelKit references (`kitfile.go:214-217`); permit only local paths for model parts, code, docs, prompts, and MCP servers (`kitfile.go:228-246`, `IsModelKitReference` false). Permit dataset `remotePath` values only for `S3PathType` or `ModelReferencePathType` (`kitfile.go:256-275`); require `remoteHash` for S3 (`"remoteHash is required when remote dataset paths are used"`), emit warning and ignore it for ModelKit references (`"remoteHash is ignored for ModelKit references in datasets"`), and reject `remoteHash` without `remotePath` (`"remote hash is only applicable when remotePath is set"`). Covers `remote-dataset*-.yaml` and `http-path-for-*.yaml` fixtures.
6. Limit a non-empty model-part `type` to `^[\w][\w.-]*$` and max 64 bytes (`modelPartTypeRegexp`, `modelPartTypeMaxLen:12-13,236-240`). Model the field as `kind` in Rust with `#[serde(rename = "type")]`. Covers `invalid-model-part-type*.yaml`.
7. MCP servers: require `name` (`"mcpServer with path %s must have a name"`), enforce uniqueness (`"duplicate mcpServer name %s"`), require local `.mcpb` suffix (`strings.HasSuffix(..., ".mcpb")` → `"path must point to a single .mcpb file"`), only `LocalPathType` permitted. Do not inspect ZIP or duplicate `manifest.json` (`MCPServer` comment `kitfile.go:108-118` + `kitfile.md:48`). Covers `mcpserver-*.yaml`.
8. LayerInfo integrity: when present, validate `digest`/`diffId` as OCI digests. Reproduce `KitfileHasLayerInfo` (`util.go:226-280`) as two independent all-or-none checks across locally materialized model/model-part, dataset, code, docs, and prompt layers; model references and remote datasets are excluded. The pinned helper also omits MCP servers even though `MCPServer` embeds `LayerInfo`; record that discrepancy and validate MCP identity according to the published resolved-config model rather than silently copying the omission. Authoring YAML carrying identity is rejected by the codec before semantic validation.

Keep validation separate from download policy: GGUF extensions, supported compression, layer/media-type matching, archive safety, remote access, and model selection remain `acorn-lib` concerns.

Exit condition: one invalid fixture can report multiple stable, path-addressed, sorted issues (e.g., combined duplicate + absolute + remote-collision), and tests document every intentional difference from KitOps v1.15.0 behavior at that pin.

### 4. Migrate the existing OCI consumer

After the portable contract is green, replace `ModelKitConfig` and `ModelKitModel` in `io/oci.rs` with `from_resolved_json` and the public model:

1. Verify the config blob's descriptor size and digest before decoding, as owned by the OCI interoperability plan.
2. Parse and validate the complete resolved config, then apply the command-specific requirement that `model` exists.
3. Select model/model-part paths and layer identities through pure adapter functions. Do not make `acorn-schema` depend on `OciLayer`, ORAS, filesystems, or GGUF.
4. Retain unrelated valid content in the parsed config for inspection and diagnostics, but do not materialize it implicitly during `download model`.
5. Map unsupported schema versions, invalid Kitfiles, missing model content, and layer-association failures to distinct diagnostics.

Exit condition: the existing ModelKit download tests use the public schema, model-free kits parse successfully before command-policy rejection, and the private two-field DTOs are deleted.

## Validation

### Portable schema and codec tests (mirror KitOps `pkg/artifact/testdata`)

- Minimal fixtures for each permitted content section and one full Kitfile containing every field from the pinned v1.15.0 types (`KitFile`/`Package`/`Model`/`ModelPart`/`DataSet`/`Code`/`Docs`/`Prompt`/`MCPServer`/`LayerInfo`); include flow-style `authors` and all `kitfile.md`-omitted fields.
- YAML and resolved-JSON golden tests, including camel-case keys, omitted optional values, arbitrary nested parameters (from `testdata/parameters/test-*.yaml` — basic, data-types, list, nested, single-value), and `digest`/`diffId` asymmetry (`yaml:"-"` vs JSON).
- Unknown fields, duplicate keys, trailing YAML documents, trailing JSON data, YAML-only parameter values (`!!js` tags), malformed numbers, and aggregate deserialization errors (`KnownFields(true)` behavior, `LoadModel:168-172`).
- Empty content, recognized/unrecognized versions (`manifestVersion: 1.0` vs `1.0.0` warning case), absolute and traversal-like paths (`filepath.Clean` + `IsAbs`), Windows and POSIX spellings, `http` URLs, ModelKit references (`IsModelKitReference` with `localhost` fallback, `:`/`@`), S3 datasets and hashes, duplicate/overlapping paths, nesting inside `remoteDatasetPaths`, model-part `^[\w][\w.-]*$`/64-byte, MCP `.mcpb` suffix and duplicate names — cover every `testdata/validation/*.yaml` fixture family (`duplicate-path-in-kitfile_*`, `http-path-for-*`, `s3-path-for-*`, `modelkit-ref-for-*`, `remote-dataset-*`, `invalid-model-part-type*`, `mcpserver-*`).
- `schemars` snapshot tests plus `cargo check -p acorn-schema --no-default-features --features alloc` and the existing WASM package check (`cargo check -p acorn-lib --no-default-features --features wasm --target wasm32-unknown-unknown`).

### Differential and consumer tests

- Generate fixtures with the pinned `kit` binary at the same commit, compare ACORN acceptance and diagnostics with `kit inspect` or `kit pack` (`kitfile_test.go:TestValidateKitfile`/`TestParameterMarshalUnmarshal` pattern), and record intentional differences such as the documented non-empty-content rule.
- Decode a config blob taken from a producer-generated ModelKit and prove that every resolved layer identity survives JSON round-trip and disappears from YAML projection (`MarshalToJSON`/`MarshalToYAML` symmetry).
- Prove `Parameters` numeric and map normalization (`0xFF->255`, sorted keys) is preserved if kept, or document divergence.
- Run focused `acorn-schema` ModelKit tests and `acorn-lib` OCI tests, then `make lint`, `make test`, `make check-wasm`, and `make audit`.

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
- The authoring Kitfile format reference defines the YAML shape (`manifestVersion`, `package`, `code`, `datasets`, `docs`, `prompts`, `mcpServers`, `model` with `parts`/`parameters`, MCP `.mcpb` verbatim raw layer `application/vnd.kitops.modelkit.mcpb.v1.raw`): <https://github.com/kitops-ml/kitops/blob/main/pkg/artifact/kitfile.md>.
- The Kitfile overview states that `manifestVersion` and at least one content section are required, paths are relative, and a kit may contain no model: <https://kitops.org/docs/kitfile/kf-overview/>.
- The Kitfile format documents package, model, model parts, datasets, code, docs, prompts, MCP Bundles, and JSON-compatible parameter values (with string/number/map caveats): <https://kitops.org/docs/kitfile/format/>.
- The pinned producer types, `LoadModel`, `MarshalToYAML`/`MarshalToJSON`, `Validate`, `GetPathType`, `IsModelKitReference`, `ParseReference`, and `KitfileHasLayerInfo` behavior are defined by KitOps v1.15.0 at commit [`6b8162ae5da4d46f1d2af2beb43e7fb077f052f4`](https://github.com/kitops-ml/kitops/blob/6b8162ae5da4d46f1d2af2beb43e7fb077f052f4/pkg/artifact/kitfile.go) and [`util.go`](https://github.com/kitops-ml/kitops/blob/6b8162ae5da4d46f1d2af2beb43e7fb077f052f4/pkg/artifact/util.go). Test fixtures proving the contract are at `pkg/artifact/testdata/validation/*.yaml` and `pkg/artifact/testdata/parameters/*.yaml`, exercised by `kitfile_test.go` (`TestValidateKitfile`, `TestParameterMarshalUnmarshal`, `TestCollectLicenses`).
- KitOps' TypeScript types independently expose the public Kitfile and OCI manifest shapes, but currently omit newer MCP-server fields; this is evidence for pinning the Go producer source rather than merging unversioned documentation ad hoc: <https://kitops.org/docs/kitops-ts/types/>.
- ACORN's implementation is in `crates/acorn-schema/src/schema/modelkit/`; producer provenance is recorded beside `tests/fixtures/modelkit/`, and `crates/acorn-lib/src/io/oci.rs` consumes the public resolved representation.
