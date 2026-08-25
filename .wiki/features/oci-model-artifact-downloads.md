---
okf_version: "0.1"
type: Plan
title: OCI model artifact downloads
description: Improve portable OCI model downloads with ModelPack interoperability, typed layer planning, index resolution, and cross-platform registry validation.
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

Advance the implemented `acorn download model oci://...` path from its initial annotated-artifact and KitOps compatibility support to standards-based CNCF ModelPack interoperability. Preserve digest-pinned identity, safe materialization, offline behavior, indirect credentials, Harbor enrichment, and existing Hugging Face routing while improving multi-layer planning and registry portability.

The remaining scope is pull-only. Pushing, deleting, retagging, registry administration, vulnerability scanning, recursive referrer downloads, and a native replacement for ORAS remain separate capabilities.

## Implemented baseline

- Explicit `oci://` tag and digest references route through a typed `OciReference` without changing unqualified Hugging Face selectors.
- Tags resolve to immutable manifest digests during planning; dry-run output and persisted model metadata retain requested and resolved identities.
- `OciTransport` isolates the version-checked ORAS process adapter, credential input, manifest inspection, blob transfer, and digest-pinned pull behavior.
- Registry profiles support Docker-compatible credentials, environment-sourced secrets over standard input, custom CA and client certificates, and explicit development-only plain HTTP.
- Annotated OCI model artifacts accept safe recognized model-resource paths, including Ollama model-store layouts, and reject runnable container images.
- KitOps ModelKits may omit the top-level manifest media type. ACORN reads the ModelKit config, fetches only its declared gzip-compressed model layer, and safely extracts GGUF, SafeTensors, ONNX, or PyTorch weights.
- Downloads use a unique sibling staging directory, reject unsafe paths and symbolic links, validate the staged inventory, and atomically publish without leaving a partial destination.
- Harbor v2 metadata enrichment, offline rejection, doctor reporting, model filters, provenance, activity logging, and synchronization integration are implemented and documented.

## Remaining design decisions

### Artifact classification

Classify manifests before selecting content. Accepted formats must have distinct policies rather than accumulating permissive media-type exceptions:

1. **CNCF ModelPack:** require `application/vnd.cncf.model.manifest.v1+json` as `artifactType` and `application/vnd.cncf.model.config.v1+json` as the config media type.
2. **KitOps ModelKit:** recognize `application/vnd.kitops.modelkit.config.v1+json` and its declared model-layer contract.
3. **Annotated generic artifact:** retain the compatibility path only when every selected descriptor has a safe `org.opencontainers.image.title` and an accepted model-resource path.
4. **Runnable image:** reject OCI or Docker image configs even when filesystem layers happen to contain model files.
5. **Unknown artifact:** fail closed with the observed artifact, config, and layer media types.

ModelPack is the preferred portable format. ModelKit and annotated artifacts are compatibility formats, not aliases for ModelPack.

### Layers and logical files

`OciFile` currently represents both a materialized file and, for ModelKit, a compressed archive layer. Separate these concepts:

- A resolved layer records its media type, digest, compressed transfer size, artifact role, optional declared path, and extraction format.
- A logical file records its safe materialization path, expected uncompressed size when known, model-resource type, and originating layer digest.
- `OciResolution` reports selected layers and known logical files. Unknown archive inventories remain explicit instead of pretending the compressed layer size is the output file size.
- Filters and quantization constraints apply before transfer when logical paths are declared. When an archive does not declare its complete inventory, apply them after safe extraction and before atomic publication.
- Dry runs must not download weight blobs merely to enumerate an archive. They should report the selected layer, transfer size, known paths, and that final inventory validation is deferred until pull.

This separation is required for ModelPack archives, multiple weight layers, sharded weights, and accurate transfer-versus-installed-size reporting.

## Improvement plan

### Phase 1: typed artifact and layer model

1. Add an internal artifact-kind discriminator for ModelPack, ModelKit, annotated generic artifacts, and rejected runnable images.
2. Introduce resolved-layer and logical-file types, then migrate `OciResolution`, dry-run serialization, model metadata, filtering, staging validation, and fake transports.
3. Keep format classification and selection pure: manifest and config inputs produce a typed resolution without filesystem mutation.
4. Preserve backward-compatible output fields where their meaning is unchanged; add explicit transfer and installed-size fields rather than silently changing `totalSize` semantics.
5. Reject duplicate layer digests with conflicting descriptors, duplicate logical paths, unsupported extraction formats, and ambiguous archive-to-file mappings.

Exit condition: annotated artifacts and ModelKits retain their current behavior while compressed layers are no longer represented as ordinary files.

### Phase 2: CNCF ModelPack support

1. Recognize the required ModelPack manifest, artifact, and config media types exactly; reject partial lookalikes and runnable images.
2. Accept model-weight and weight-configuration roles for `download model`. Exclude code, documentation, and dataset layers unless a future command explicitly requests them.
3. Initially support these layer forms:
   - `application/vnd.cncf.model.weight.v1.raw`
   - `application/vnd.cncf.model.weight.v1.tar`
   - `application/vnd.cncf.model.weight.v1.tar+gzip`
   - the corresponding `application/vnd.cncf.model.weight.config.v1.*` forms
4. Require a safe `org.cncf.model.filepath` annotation for raw layers. Validate optional filepath and file-metadata annotations on archive layers without treating them as a complete inventory unless the format guarantees that meaning.
5. Fetch raw layers directly to their validated paths. Extract tar and gzip-tar layers with the shared bounded archive reader, rejecting traversal, links, duplicate paths, unsupported entry types, and expansion-limit violations.
6. Support multiple weight layers and preserve shard completeness. Apply model-resource policy to extracted files and require at least one accepted weight.
7. Defer `tar+zstd` until a bounded decoder, expansion accounting, fixtures, and cross-platform tests are available. Return an actionable unsupported-compression error meanwhile.
8. Add fixtures produced by both KitOps `--use-model-pack` and an independent ModelPack implementation to avoid encoding one producer's conventions as the standard.

Exit condition: ACORN can resolve, dry-run, pull, validate, and atomically publish raw, tar, and gzip ModelPacks containing single-file and sharded models without downloading unrelated artifact roles.

### Phase 3: OCI index and platform resolution

1. Decode OCI image indexes separately from image manifests and retain the selected child descriptor in resolution metadata.
2. Automatically resolve an index only when it contains exactly one eligible model artifact. Never select the first descriptor by registry order.
3. For multiple eligible descriptors, require an explicit platform or artifact choice. Add CLI/configuration surface only after fixtures demonstrate the required selection dimensions.
4. Pin the chosen child manifest by digest before fetching config or layers so mutable tags cannot race between resolution steps.
5. Reject nested indexes beyond a bounded depth, cyclic descriptor graphs, mixed runnable-image/model candidates without explicit selection, and unsupported descriptor media types.

Exit condition: single-artifact indexes resolve deterministically, ambiguous indexes fail with actionable candidates, and runnable platform images remain rejected.

### Phase 4: transport and trust hardening

1. Add cancellation coverage for manifest fetch, config fetch, blob transfer, and archive extraction; interrupted work must remove only the exact staging directory.
2. Map authentication, authorization, not-found, rate-limit, TLS, digest mismatch, unsupported media type, archive-policy, and cancellation failures into stable typed diagnostics.
3. Verify downloaded blob size and digest independently when ORAS behavior or output cannot prove both properties at ACORN's validation boundary.
4. Record transfer size separately from installed size and surface archive expansion limits before extraction when metadata permits.
5. Preserve resolved digests and artifact types for a future Cosign or Notation policy hook. Signature enforcement is not part of this plan and must not be implied by digest verification.
6. Reassess a native OCI Distribution client only after measuring ORAS installation failures, startup cost, cancellation limitations, and unsupported platforms.

Exit condition: failures are typed and redacted, cancellation is recoverable, content identity is verified at the publication boundary, and digest integrity is not presented as publisher authenticity.

### Phase 5: interoperability and release validation

1. Publish small, non-secret annotated-artifact, ModelKit, ModelPack, multi-shard, and index fixtures to a disposable Distribution registry and a supported Harbor instance.
2. Test mutable tags and immutable digests, public and authenticated access, custom CA, optional mutual TLS, explicit plain HTTP, redirects, destination collisions, interrupted transfers, and corrupted blobs.
3. Add Windows and Linux registry jobs covering path handling, credential helpers, process cancellation, archive extraction, and atomic rename behavior.
4. Document the artifact classification policy, supported ModelPack roles and compression formats, deferred-inventory behavior, transfer-versus-installed sizes, and digest-versus-signature trust boundary.
5. Run focused tests, `make lint`, `make test`, `make help`, `make wasm`, `make book`, and `make audit`. Resolve or formally baseline the existing repository-wide unused-dependency audit findings before completion.

Exit condition: the supported format matrix passes on Windows and Linux against generic OCI Distribution and Harbor, documentation matches observed behavior, and the plan can move to `complete`.

## Validation strategy

### Unit and fixture tests

- Classify strict ModelPack, ModelKit, annotated generic, runnable image, unknown artifact, manifest index, and malformed descriptor fixtures.
- Test raw, tar, gzip-tar, unsupported zstd, multiple weight layers, weight-configuration layers, missing models, and excluded code/documentation/dataset roles.
- Validate filepath and file-metadata annotations, safe and unsafe archive entries, duplicate paths, symlinks, expansion limits, digest/size mismatches, and cleanup.
- Test pre-transfer and post-extraction filters, quantization selection, shard completeness, known and deferred inventories, and transfer/installed-size reporting.
- Cover fake-ORAS success and every typed failure without exposing credentials or depending on experimental formatted output.
- Preserve existing OCI reference, registry-profile, Harbor, ModelKit, Ollama-layout, offline, provenance, and synchronization tests.

### Integration tests

- Pull equivalent ModelPacks produced by KitOps and an independent implementation, by both tag and digest, and compare their validated output inventories.
- Pull public and authenticated fixtures from generic Distribution and Harbor registries.
- Exercise single-candidate and ambiguous OCI indexes, multi-layer and multi-shard models, raw and archived weights, interrupted and corrupted transfers, and destination collisions.
- Verify `--dry-run`, `--raw`, `--filter`, `--ignore`, `--quantization`, `--local-dir`, whitelist, configuration, activity logging, and `--sync` semantics across all accepted OCI formats.

## Risks and mitigations

- **ModelPack annotations are optional or incomplete.** Represent unknown archive inventories honestly, defer file filtering until safe extraction when required, and never download weight blobs during dry-run solely for discovery.
- **Layer media types may be mistaken for file media types.** Keep artifact, layer, archive, and logical-file types distinct and validate each boundary independently.
- **Archives can expand unexpectedly or escape the destination.** Reuse bounded extraction, reject links and unsafe entries, validate the final inventory, and publish only through the existing staging boundary.
- **Multiple producers may interpret the specification differently.** Require fixtures from two implementations and follow required ModelPack discriminators rather than producer-specific annotations.
- **Indexes can hide runnable images beside model artifacts.** Select only explicitly eligible model descriptors and fail closed on ambiguity.
- **Mutable tags weaken reproducibility.** Resolve and persist immutable digests before config or blob retrieval.
- **Digest integrity is not publisher authenticity.** Keep the distinction explicit and reserve signature policy for separate work.
- **ORAS and registry behavior can drift.** Pin a minimum ORAS version, parse stable manifest JSON, retain typed process boundaries, and run live interoperability tests.

## Remaining acceptance criteria

- Existing annotated OCI and KitOps downloads remain backward compatible and continue to reject runnable container images.
- ModelPack artifact and config types are validated strictly; only model-weight and weight-configuration roles are materialized.
- Raw, tar, and gzip-tar ModelPacks support safe single-file, multi-file, and sharded weights; unsupported zstd layers fail clearly.
- Resolution distinguishes transfer layers from logical output files and reports known versus deferred archive inventories accurately.
- Filters, quantization, shard validation, staging cleanup, digest verification, provenance, dry-run, and atomic publication work across every accepted format.
- Single-candidate OCI indexes resolve deterministically and ambiguous indexes require an explicit choice.
- Public, authenticated, custom-CA, interruption, corruption, and collision scenarios pass against generic Distribution and Harbor fixtures.
- Windows and Linux interoperability jobs pass, documentation matches the supported matrix, and repository validation has an accepted result.

## Evidence

- The implemented baseline is centered in `crates/acorn-lib/src/io/oci.rs`, `crates/acorn-cli/src/commands/download/model/plan.rs`, `crates/acorn-lib/src/io/config.rs`, and `crates/acorn-lib/src/io/api/harbor.rs`.
- Local validation after KitOps compatibility support: `make lint`, `make test` with 1,251 passing tests and 15 skipped tests, and `make book` succeeded. macOS emitted sandbox-only `xcrun` cache warnings without changing successful exit statuses.
- Live GHCR validation resolved Jozu Phi-3 and MiniLM ModelKits; an end-to-end MiniLM pull fetched, safely extracted, inventoried, and atomically published `model.safetensors` before the temporary fixture was removed.
- The CNCF ModelPack specification requires `application/vnd.cncf.model.manifest.v1+json`, `application/vnd.cncf.model.config.v1+json`, and defined weight/config layer roles: <https://github.com/modelpack/model-spec/blob/main/docs/spec.md>.
- ModelPack defines `org.cncf.model.filepath` and file-metadata layer annotations: <https://github.com/modelpack/model-spec/blob/main/docs/annotations.md>.
- KitOps can produce ModelPack artifacts with `kit pack --use-model-pack`, providing one interoperability fixture source: <https://kitops.org/docs/integrations/modelpack/>.
