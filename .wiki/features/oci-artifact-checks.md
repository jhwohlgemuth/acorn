---
okf_version: "0.1"
type: Plan
title: Add OCI model artifacts as a checkable standard
description: Make remote OCI model artifacts a first-class ACORN check target by reusing the existing manifest resolver without implicitly downloading model weights.
difficulty: medium
impact: high
package: [acorn-schema, acorn-analysis, acorn-lib, acorn-cli]
tags: [feature, check, oci, modelkit, modelpack, oras, validation]
status: planned
priority: next
---

# Goal

Add `oci` to ACORN's standard selector and support manifest-level checks of remote model artifacts:

```console
acorn check oci://registry.example.org/models/qwen:latest --standard oci
acorn check oci://registry.example.org/models/qwen@sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef --standard oci
```

An unambiguous `oci://` check source also selects the OCI standard when the caller leaves the existing default standard in place. The checker resolves tags to immutable manifest digests, validates the registry descriptor, index or manifest, configuration blob, package convention, and declared model inventory, and reports findings through the existing schema and quality categories.

The first implementation is an **OCI model-artifact profile**, not a claim of generic OCI conformance. It supports the artifact forms already recognized by ACORN: annotated GGUF artifacts, KitOps ModelKits, and CNCF ModelPacks. Runnable container images and unsupported artifact types receive deterministic findings.

# Current evidence

- `crates/acorn-lib/src/io/oci.rs` already parses and validates explicit OCI references, descriptors, schema-version-2 manifests and indexes, descriptor digests and sizes, safe layer paths, ModelKit configuration, ModelPack configuration and media types, model selection, and immutable resolution output.
- `OrasTransport::resolve` fetches descriptor, manifest or index, and configuration data without transferring selected model-weight layers. `OrasTransport::pull` owns later materialization, blob verification, archive extraction, GGUF inspection, and atomic publication.
- OCI resolution already distinguishes annotated artifacts, ModelKits, ModelPacks, runnable images, invalid ModelPack marker combinations, and unsupported artifacts.
- The I/O test suite has focused coverage for reference parsing, exact configuration-blob hashing, media-type classification, fake-ORAS resolution, ModelKit and ModelPack behavior, index resolution, path safety, runnable-image rejection, authentication arguments, and minimum ORAS versions.
- `acorn check` currently accepts local paths and HTTP(S) sources, then routes filesystem paths through the path-based analyzer service. Its preparation boundary rejects `oci://` sources before OCI resolution can run.
- The analyzer standard enum and dispatch use exhaustive matches. Adding an OCI variant therefore requires deliberate behavior in checking, crosswalk, diff, formatting, and schema-export paths.
- The completed portable analysis and service extraction moved the canonical `Standard` selector and aliases to `acorn-schema`.
- Documentation already describes OCI Image and Distribution support for model downloads, but OCI is not listed as a checkable standard.

# Scope

## Included

- A canonical `Standard::Oci` value with `oci` as its CLI spelling and serialized/display name.
- Explicit `--standard oci` selection and automatic selection for an unambiguous `oci://` check source when the default standard remains selected.
- Tag and digest references for public or configured authenticated registries.
- One ORAS resolution per check execution, shared by schema and quality reporting.
- Schema findings for reference, registry descriptor, index, manifest, configuration-blob, descriptor, digest, size, media-type, and safe-path failures.
- Quality findings for artifact classification, ModelKit or ModelPack semantic consistency, declared inventory, and primary-model selection.
- Successful informational checks under `--all`, including requested reference, resolved immutable digest, package format, inventory confidence, and selected or ignored layer counts.
- Existing human, terse, raw/CSV, and JSON check output and ordinary `--no-fail` behavior.
- Registry transport options resolved from ACRE without adding secrets to files or command-line arguments.
- Offline rejection before ORAS or a registry is contacted.

## Excluded

- Implicit transfer of model-weight blobs. A routine metadata check must not download a potentially very large model.
- Verification of selected layer bytes or GGUF contents. Those checks remain part of materialization through `OrasTransport::pull`; a future explicit deep-check plan may expose them.
- Local OCI image-layout directories, OCI-layout archives, Docker save archives, or container-engine image stores.
- Generic validation of arbitrary OCI artifacts, runnable images, signatures, attestations, SBOMs, referrers, or publisher identity.
- OCI schema export. OCI checking spans references, distribution responses, manifests, indexes, configuration blobs, and an ACORN model profile; it is not one generated JSON Schema.
- OCI formatting, semantic diff, crosswalks, FAIR assessment, prose, readability, compute, link checks, fix suggestions, or Rhai workflow hooks.
- New registry clients or a replacement for ORAS.

# Project boundaries

The portable analysis and service boundary has landed. Reconfirm exact paths against that baseline before implementation; the intended ownership is:

- **Create:** a host-side OCI check adapter under `crates/acorn-lib/src/analyzer/host/` and focused adapter tests when the post-boundary module layout is known.
- **Update:** `crates/acorn-schema/src/standard/mod.rs` for the canonical selector; `crates/acorn-analysis/src/check.rs` for exhaustive portable behavior; `crates/acorn-lib/src/io/oci.rs` for a single-pass, stage-tagged inspection boundary reused by download resolution; `crates/acorn-lib/src/analyzer/host/{runtime,service}/mod.rs` for supported categories and host dispatch; `crates/acorn-cli/src/cli/{arguments,parse}.rs`, `crates/acorn-cli/src/commands/check/mod.rs`, and their focused tests for source selection, option policy, and rendering; command implementations with exhaustive standard matches only to return explicit unsupported errors; `docs/src/concepts/standards.md`, `docs/src/cli-reference/commands/check.md`, and `CommandLineHelp.md` for the user contract.
- **Retain:** OCI transfer and atomic publication in `OrasTransport::pull`; existing ModelKit and ModelPack schema owners; ACRE's credential-environment policy; the current check categories and report shapes.
- **Do not update:** model inference policy, database schemas, sidecar artifact handling, crosswalk mappings, portable service operations, or WASM bindings.

Before implementation, replace the host-adapter directory placeholder and exhaustive-match qualification with the exact post-service-boundary file list. The implementation owner holds the analyzer/check, CLI contract, transfer/OCI, schema-root, and documentation-generation locks for the integration commit.

# Dependencies and ordering

1. **Satisfied:** the portable analysis and service boundary is retired; OCI uses the final `Standard` owner and host-analysis composition.
2. Reverify `OrasTransport::resolve` against [Model acquisition and metadata hardening](model-acquisition-hardening.md). If that plan still owns `io/oci.rs`, serialize the stage-tagging change with its integration owner; do not fork manifest or package policy.
3. Land OCI checks before [CLI composition boundary](../refactors/core/cli-composition-boundary.md) migrates the same command contract, so the later composition work moves one settled source type and exit policy.
4. Regenerate command help once after CLI syntax and descriptions stabilize.

# Decisions

## Check target and source policy

The canonical source is one explicit `oci://registry/repository:tag` or `oci://registry/repository@sha256:...` reference. The check command treats it as a remote artifact target rather than coercing it into a temporary file or filesystem path.

`--standard oci` rejects local paths, HTTP(S) URLs, pasted or piped content, branch or commit selection, merge-request selection, watch mode, file filters, and ignore patterns. An `oci://` source with a conflicting non-OCI standard fails clearly instead of falling back to RAD.

The CLI preparation value becomes source-aware, with path inputs and OCI targets represented separately. OCI execution bypasses path classification, source-document indexing, configured RAD workflow hooks, FAIR profile construction, and fix proposal generation.

## One inspection, two categories

Add a narrow OCI inspection result that preserves the existing `OciResolution` on success and tags artifact failures by stage. `OrasTransport::resolve` delegates to the same internal inspection steps so downloads and checks cannot drift.

- **Schema:** reference grammar; top-level descriptor; schema-version-2 index or manifest; accepted OCI/Docker descriptor media types where already supported; nested-index and ambiguity rules; descriptor digest and size syntax; exact configuration-blob size and digest; safe declared paths.
- **Quality:** supported model-artifact classification; ModelKit and ModelPack marker agreement; package-specific layer identities and roles; one complete primary GGUF group based on declared inventory; explicit selected and ignored components.

Do not classify errors by matching rendered strings. The inspection boundary carries a private or crate-visible stage enum that the host adapter converts into `CheckCategory::Schema` or `CheckCategory::Quality`.

If schema inspection fails, quality receives an unsupported/skipped status caused by the schema failure rather than issuing a second registry request. Successful runs emit one success check for each supported category; ordinary output hides them unless `--all` is selected.

Missing ORAS, an unsupported ORAS version, absent credentials, TLS/configuration failures, and network unavailability are execution failures rather than claims that the artifact is invalid. They return a nonzero command error with redacted context. Artifact responses that were retrieved and found malformed become checks.

## Manifest-level depth

The default check may fetch:

- the registry descriptor;
- one index and its eligible child manifest when indirection is present;
- the resolved manifest; and
- the small configuration blob needed to validate ModelKit or ModelPack structure.

It does not invoke `oras pull`, fetch selected weight layers, create a model destination, inspect actual GGUF bytes, or claim that descriptor integrity proves publisher authenticity. Fake-ORAS tests fail if the checker attempts a pull or requests a declared weight blob.

## Registry configuration and offline behavior

Public registries use default `OciTransportOptions`. When ACRE defines registry profiles, normalize each profile endpoint to its registry authority and select the unique profile matching the OCI reference. No match uses default public transport options; more than one match is a configuration error. Existing environment-backed credential, username, registry-config, CA, client-certificate, client-key, and development `plainHttp` behavior remains unchanged.

`--offline` rejects an OCI target before ORAS construction or registry access. It does not treat cached download metadata as a fresh conformance result.

## Standard behavior outside `check`

The shared selector means other commands can parse `oci`; each command must then report its actual capability:

- `export --schema --standard oci` fails with a clear message that OCI checking has no single generated schema.
- format, diff, crosswalk, enrichment, and ordinary export reject OCI as unsupported.
- standard detection does not infer OCI from arbitrary JSON objects because a manifest alone lacks registry, blob, and package-resolution context.
- OCI supports only schema and quality check categories in this plan.

## Analyzer synchronization and output

Analyzer synchronization becomes capability-driven: selecting a standard that does not support prose must not install or synchronize Vale. This is required for OCI and removes an unnecessary external action for other schema-only standards when it can be done without changing their results.

OCI reports use the requested reference as the logical URI. Success metadata and diagnostics redact credentials and never include environment-variable contents, registry tokens, configuration file contents, or ORAS command lines containing sensitive values.

# Implementation sequence

1. On the post-service-boundary baseline, add `Standard::Oci`, its parser alias, exhaustive unsupported behavior, and selector tests without enabling execution.
2. Refactor the existing OCI resolver into one stage-tagged inspection pipeline while preserving download resolution and all existing OCI tests.
3. Add the host OCI check adapter that converts one inspection into schema and quality category results.
4. Add source-aware check preparation and execution for `oci://` targets, including incompatible-option, offline, hook, and suggestion policy.
5. Resolve the unique matching ACRE registry profile and cover public, authenticated, custom-CA, and `plainHttp` transport argument behavior through existing redaction boundaries.
6. Add end-to-end fake-ORAS CLI tests proving one resolution, no weight transfer, deterministic categories, output formats, and exit policy.
7. Update standards and check documentation, regenerate command help, and run focused followed by full repository gates.

# Validation

## Focused tests

- Standard parsing accepts `oci` and every other command returns its documented unsupported result.
- An `oci://` check target is kept as a remote target, not normalized as a filesystem path.
- Canonical tag and digest references resolve; malformed, traversal-shaped, ambiguous, query-bearing, and fragment-bearing references fail before ORAS.
- Offline mode invokes neither ORAS nor a registry.
- A valid annotated artifact, ModelKit, raw ModelPack, archived ModelPack, and one-candidate index each produce successful schema and quality checks.
- Malformed descriptors, manifests, indexes, config blobs, layer paths, ModelKit identities, ModelPack markers, and primary-model inventories produce the correct category without string-based classification.
- Runnable images and unsupported artifacts have deterministic quality outcomes.
- The fake transport records one inspection per check and rejects `pull` or weight-blob access.
- Public registry access uses default options; a unique matching ACRE profile supplies existing ORAS options; ambiguous matches and missing credential environments fail safely.
- `--all`, `--terse`, `--raw`, `--output json`, and `--no-fail` retain existing report and exit semantics.
- OCI rejects paste/stdin, local and HTTP sources, watch, repository revision modes, file filters, Rhai hooks, and fix suggestions with actionable errors.
- OCI selection does not synchronize Vale.

## Regression and repository gates

- Existing OCI download, ModelKit, ModelPack, GGUF, sidecar, sync, and registry-profile tests remain unchanged in behavior.
- Existing standards retain their parser, detection, schema export, diff, format, crosswalk, and check behavior.
- Run focused tests for `acorn-schema`, `acorn-analysis`, OCI I/O, analyzer host service/runtime, CLI parsing, and the check command.
- Run `make lint`, `make test`, `make check-portable`, `make check-wasm`, `make doc`, and the documentation build after focused tests pass.

# Acceptance criteria

- `acorn check <oci-reference> --standard oci` validates a supported remote OCI model artifact and reports through the existing schema and quality categories.
- A valid tag result records and displays its resolved immutable digest under `--all`.
- The checker performs one manifest-level inspection and never transfers selected model-weight layers or creates a model installation destination.
- Schema and quality diagnostics come from explicit inspection stages, not rendered-message matching.
- Malformed retrieved artifacts are findings; missing tools, credentials, TLS configuration, offline permission, and network availability are redacted execution errors.
- Registry profiles are selected deterministically by authority without storing or printing secrets.
- Only schema and quality are advertised for OCI; all unrelated commands and categories reject or report unsupported behavior explicitly.
- OCI checking is host-only and does not add ORAS, filesystem, process, or network dependencies to `acorn-schema`, `acorn-analysis`, `acorn-service`, or WASM builds.
- The standards and check documentation distinguish manifest-level validation from deep blob/GGUF verification and from publisher authenticity.
- Full tests, lint, portable/WASM checks, rustdoc, generated help, and book documentation pass.
