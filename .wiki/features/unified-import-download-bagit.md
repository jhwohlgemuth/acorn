---
okf_version: "0.1"
type: Plan
title: Unified import and download with BagIt archives
description: Finish the shared bucket transfer pipeline and provide inferred ZIP, TAR, TAR.GZ, and 7z BagIt import and export.
difficulty: high
impact: high
package:
  - acorn-lib
  - acorn-cli
tags:
  - import
  - bagit
  - archive
  - cli
status: in-progress
---

# Unified import and download with BagIt archives

## Goal

Support local directories, remote repositories, and verified BagIt archives through `acorn import`, while retaining remote-only bucket download and distinct model metadata/weight executors.

Complete archive-format parity by adding 7z inference, creation, extraction, CLI selection, and BagIt round trips without weakening extraction safety or the WASM feature boundary.

## Current evidence

- `MimeType` recognizes and infers semantic document containers before generic ZIP, TAR, and gzip signatures.
- Archive creation and guarded extraction share an `ArchiveFormat` implementation for 7z, ZIP, TAR, and TAR.GZ; inference reads at most 8 KiB.
- Bucket import and download call one transfer executor with explicit local/offline policy.
- Model commands flatten shared source, resolution, and synchronization argument groups into distinct executors.
- `sevenz-rust2` 0.21 is an optional, maintained pure-Rust Apache-2.0 dependency confined to the `std` feature; it replaces the vulnerable, unmaintained `sevenz-rust` line.
- Extraction rejects traversal, unsupported TAR entries, duplicate paths, excessive entry/expanded-size limits, and pre-existing symlink components; archive creation rejects outputs inside the source tree.

## Recommendation

Finish the existing implementation before declaring the feature complete. Keep archive dispatch behind `MimeType` and `ArchiveFormat`, keep bucket transfer policy in the shared executor, and extend those same paths for 7z rather than adding format-specific CLI branches. Select a maintained pure-Rust 7z implementation that can be optional and confined to `acorn-lib/std`; document the dependency and license decision before integrating it.

## Outstanding work

1. Complete archive and MIME coverage.
   - Add semantic-container precedence tests for EPUB and OpenDocument formats, plus generic TAR inference, unknown bytes, and manual override tests for every extractor.
   - Add malformed TAR and TAR.GZ cases and security tests for absolute paths, traversal, links, devices, unsupported entries, duplicate destinations, and file/directory collisions.
   - Confirm archive rejection occurs before the destination is mutated.
2. Finish 7z integration coverage.
   - Add malformed input, manual override, remote BagIt import, and format-specific extraction-security tests beyond the common round-trip and path-safety coverage.
3. Finish bucket integration coverage.
   - Test remote ZIP, TAR, TAR.GZ, and 7z streaming, temporary-path cleanup on success and failure, and preservation of the original URI as provenance.
   - Test configuration expansion, mixed local/remote offline rejection, filters, ignores, flattening, clobbering, counts, database ingestion, and activity logging through `acorn import`.
   - Test that top-level download rejects local sources and offline mode while `download model` retains its existing local and offline behavior.
4. Finish BagIt export and validation coverage.
   - Cover root and single-enclosing-directory bags for every supported archive format.
   - Cover SHA-256 and SHA-512 manifests, malformed and duplicate records, missing and unlisted payloads, checksum mismatches, and unsupported manifest algorithms.
   - Test suffix-selected defaults, missing-suffix ZIP defaults, explicit overrides, suffix conflicts, output cleanup after failure, and re-importability for ZIP, TAR, TAR.GZ, and 7z.
5. Complete model consolidation cleanup.
   - Add equivalence tests proving import and download produce identical resolution plans from inline selectors, model-list files, configured entries, fallback settings, interactive selection, search limits, and popularity thresholds.
   - Migrate remaining tests to typed request structures, then remove test-only long-signature wrappers without changing model-specific flags, defaults, synchronization, database behavior, or output.
6. Expand production-path regression coverage after removal of the test-only model and bucket-transfer wrappers.
7. Finish documentation and release validation.
   - Regenerate CLI help and update import/export/download documentation with 7z behavior, suffix rules, manual overrides, remote handling, and security restrictions.
   - Update this concept's evidence and acceptance criteria after the outstanding tests land.
   - Make the repository's standard `make wasm` target pass with the configured Rust toolchain instead of relying on an explicit compiler override.
   - Resolve, upgrade, or deliberately document the currently reported RustSec findings so `make audit` has an accepted result.
   - Run focused tests, `make lint`, `make test`, `make help`, `make wasm`, and `make audit`.

## Validation

- 2026-08-21: `make lint`, `make test` (1,211 passed), `make help`, and `make wasm` passed.
- 2026-08-21: `make audit` completed dependency compilation but remains blocked by the existing `cargo-udeps` findings for `acorn-cli/keyring` and `acorn-lib/rmcp`. A direct RustSec scan prompted replacement of vulnerable `sevenz-rust` with maintained `sevenz-rust2`; the replacement has no reported advisory.
- Exercise MIME precedence, unknown input, ZIP/TAR/TAR.GZ/7z round trips, traversal rejection, and complete BagIt manifests.
- Exercise local and remote import policy, provenance, filtering, flattening, clobbering, and database ingestion.
- Exercise BagIt output suffix inference, explicit overrides, conflicts, and re-import.
- Run `make lint`, `make test`, `make help`, `make wasm`, and `make audit`.

## Acceptance criteria

- ZIP, TAR, TAR.GZ, and 7z BagIt archives round-trip through export and import.
- Unsafe archive entries and incomplete or incorrect manifests fail before payload ingestion.
- Top-level download rejects local/offline bucket requests without changing model behavior.
- CLI help and user documentation describe the supported formats and policy split.
- The standard lint, test, help, WASM, and audit targets have accepted results.
