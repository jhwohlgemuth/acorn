---
okf_version: "0.1"
type: Plan
title: OCI model artifact downloads
description: Improve portable GGUF-focused OCI downloads with ModelPack interoperability, typed layer planning, index resolution, and cross-platform registry validation.
difficulty: high
impact: high
package:
  - acorn-lib
  - acorn-cli
tags:
  - oci
  - oras
  - harbor
  - model
  - modelpack
  - kitops
  - download
status: in-progress
---

# OCI model artifact downloads

## Goal

Advance the implemented `acorn download model oci://...` path from its initial annotated-artifact and KitOps compatibility support to standards-based CNCF ModelPack interoperability with a GGUF-focused payload boundary. Preserve digest-pinned identity, safe materialization, offline behavior, indirect credentials, Harbor enrichment, and existing Hugging Face routing while improving multi-layer planning and registry portability.

`MimeType::is_model_resource` remains the shared broad classification helper for GGUF, SafeTensors, ONNX, PyTorch, and model-support files. OCI acceptance must use a separate GGUF policy instead of narrowing or renaming that helper. The OCI path promises validated GGUF content, not compatibility with a particular llama.cpp release or runtime.

The remaining scope is pull-only. Pushing, deleting, retagging, registry administration, vulnerability scanning, recursive referrer downloads, and a native replacement for ORAS remain separate capabilities.

## Remaining work

### Transport and trust hardening

1. Add cancellation coverage for manifest fetch, config fetch, blob transfer, and archive extraction; interrupted work must remove only the exact staging directory.
2. Map authentication, authorization, not-found, rate-limit, TLS, digest mismatch, unsupported media type, archive-policy, and cancellation failures into stable typed diagnostics.
3. Surface archive expansion limits before extraction when descriptor or file-metadata annotations provide reliable uncompressed sizes.
4. Preserve resolved digests and artifact types for a future Cosign or Notation policy hook. Signature enforcement is not part of this plan and must not be implied by digest verification.
5. Reassess a native OCI Distribution client only after measuring ORAS installation failures, startup cost, cancellation limitations, and unsupported platforms.

Exit condition: failures are typed and redacted, cancellation is recoverable, content identity is verified at the publication boundary, and digest integrity is not presented as publisher authenticity.

### Interoperability and release validation

1. Publish small, non-secret annotated-artifact, ModelKit, ModelPack, split-GGUF, projector, and index fixtures to a disposable Distribution registry and a supported Harbor instance.
2. Test mutable tags and immutable digests, public and authenticated access, custom CA, optional mutual TLS, explicit plain HTTP, redirects, destination collisions, interrupted transfers, and corrupted blobs.
3. Add Windows and Linux registry jobs covering path handling, credential helpers, process cancellation, archive extraction, and atomic rename behavior.
4. Add producer-derived fixtures for strict media-type matching, uncompressed TAR, multiple layers, split metadata, projector association, ambiguous indexes, corruption, and cancellation. The current fake-ORAS suite covers annotated GGUF, ModelKit GGUF, raw and gzip ModelPack, single-candidate indexes, and GGUF/non-GGUF policy boundaries.
5. Run `make test`, `make wasm`, `make book`, and `make audit` after focused tests, formatting, workspace checks, lint, generated help, and live ModelKit resolution have passed. Resolve or formally baseline existing repository-wide unused-dependency audit findings before completion.

Exit condition: the supported format matrix passes on Windows and Linux against generic OCI Distribution and Harbor, documentation matches observed behavior, and the plan can move to `complete`.

## Remaining validation

### Unit and fixture tests

- Add producer-derived uncompressed-TAR, multiple-layer, split-metadata, projector, partial-ModelPack, ambiguous-index, corrupted-blob, and interrupted-transfer fixtures.
- Complete stable typed-error coverage for authentication, authorization, not-found, rate-limit, TLS, media policy, archive policy, corruption, and cancellation without exposing credentials.
- Preserve the focused fake-ORAS and GGUF parser coverage while adding cancellation cleanup assertions.

### Integration tests

- Pull equivalent ModelPacks produced by KitOps and an independent implementation, by both tag and digest, and compare their validated output inventories.
- Pull public and authenticated fixtures from generic Distribution and Harbor registries.
- Exercise ambiguous OCI indexes, multi-layer and split GGUF models, projector associations, raw and archived weights, interrupted and corrupted transfers, and destination collisions.
- Verify `--dry-run`, `--raw`, `--filter`, `--ignore`, `--quantization`, `--local-dir`, whitelist, configuration, activity logging, and `--sync` semantics across all accepted OCI formats.

## Risks and mitigations

- **ModelPack annotations are optional or incomplete.** Represent unknown archive inventories honestly, defer file filtering until safe extraction when required, and never download weight blobs during dry-run solely for discovery.
- **Generic model-resource classification may be mistaken for OCI acceptance.** Keep `is_model_resource` broad and route OCI payloads through a separate GGUF policy with explicit tests for the distinction.
- **Layer media types may be mistaken for file media types.** Keep artifact, layer, archive, and logical-file types distinct and validate each boundary independently.
- **Archives can expand unexpectedly or escape the destination.** Reuse bounded extraction, reject links and unsafe entries, validate the final inventory, and publish only through the existing staging boundary.
- **Multiple producers may interpret the specification differently.** Require fixtures from two implementations and follow required ModelPack discriminators rather than producer-specific annotations.
- **Indexes can hide runnable images beside model artifacts.** Select only explicitly eligible model descriptors and fail closed on ambiguity.
- **Mutable tags weaken reproducibility.** Resolve and persist immutable digests before config or blob retrieval.
- **Digest integrity is not publisher authenticity.** Keep the distinction explicit and reserve signature policy for separate work.
- **ORAS and registry behavior can drift.** Pin a minimum ORAS version, parse stable manifest JSON, retain typed process boundaries, and run live interoperability tests.

## Remaining acceptance criteria

- Producer-derived ModelPack fixtures confirm raw, uncompressed-TAR, and gzip-TAR behavior across multiple layers, split GGUFs, and projector association.
- ORAS and extraction cancellation terminate cleanly and remove only ACORN's exact staging directory.
- Registry and archive failures use stable typed, redacted diagnostics.
- Public, authenticated, custom-CA, interruption, corruption, and collision scenarios pass against generic Distribution and Harbor fixtures.
- Windows and Linux interoperability jobs pass, documentation matches the supported matrix, and repository validation has an accepted result.

## Evidence

- The implemented baseline is centered in `crates/acorn-lib/src/io/oci.rs`, `crates/acorn-cli/src/commands/download/model/plan.rs`, `crates/acorn-lib/src/io/config.rs`, and `crates/acorn-lib/src/io/api/harbor.rs`.
- The focused fake-ORAS suite covers annotated GGUF, KitOps GGUF, raw and gzip ModelPack, deferred inventories, blob digest/size checks, single-child indexes, GGUF structural validation, and the generic-model-resource versus OCI-GGUF policy boundary.
- Live GHCR dry runs on 2026-08-24 resolved `jozu-ai/phi3:3.8b-mini-instruct-4k-q4_K_M` to its immutable GGUF ModelKit digest and rejected `jozu-ai/all-minilm-l6-v2:safetensor` as outside the GGUF payload boundary before transferring weights.
- The CNCF ModelPack specification requires `application/vnd.cncf.model.manifest.v1+json`, `application/vnd.cncf.model.config.v1+json`, and defined weight/config layer roles: <https://github.com/modelpack/model-spec/blob/main/docs/spec.md>.
- ModelPack defines `org.cncf.model.filepath` and file-metadata layer annotations: <https://github.com/modelpack/model-spec/blob/main/docs/annotations.md>.
- KitOps can produce ModelPack artifacts with `kit pack --use-model-pack`, providing one interoperability fixture source: <https://kitops.org/docs/integrations/modelpack/>.
