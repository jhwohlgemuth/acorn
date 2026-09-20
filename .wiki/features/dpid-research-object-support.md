---
type: Plan
title: "Add read-only dPID research-object support"
description: "Recognize dPIDs, resolve immutable versions and manifests, import resolver RO-Crate metadata into RAD, and preserve identity and provenance boundaries without adding minting or key custody."
difficulty: high
impact: high
package: [acorn-schema, acorn-lib, acorn-cli]
tags: [feature, pid, dpid, research-object, ro-crate, json-ld, ipfs, cid, resolver, import, provenance]
status: planned
---

# Project code boundaries

- **Update:** `crates/acorn-schema/src/pid/{mod.rs,tests.rs}`, `crates/acorn-schema/src/research_activity/{mod.rs,common.rs,output.rs}`, `crates/acorn-schema/src/standard/{mod.rs,tests/mod.rs}`, `crates/acorn-lib/src/io/api/{mod.rs,tests/mod.rs}`, `crates/acorn-lib/assets/constants/application.json`, `crates/acorn-lib/src/analyzer/host/discovery/{mod.rs,tests/mod.rs}`, `crates/acorn-lib/src/io/config/transfer.rs`, `crates/acorn-cli/src/cli/{mod.rs,arguments/mod.rs,tests/mod.rs}`, `crates/acorn-cli/src/commands/{mod.rs,import/mod.rs,import/tests/mod.rs}`, `docs/src/concepts/persistent-identifiers.md`, `docs/src/cli-reference/commands/{gather,import}.md`, and generated command help.
- **Create:** `crates/acorn-schema/src/pid/dpid/{mod.rs,tests.rs}`, `crates/acorn-schema/src/standard/rocrate/mod.rs`, `crates/acorn-schema/src/standard/tests/rocrate.rs`, `crates/acorn-lib/src/io/api/dpid.rs`, `crates/acorn-lib/src/io/api/tests/dpid.rs`, `crates/acorn-cli/src/commands/import/dpid.rs`, and pinned resolver/history/RO-Crate fixtures under `tests/fixtures/dpid/`.
- **Delete:** None.

# Dependencies

The portable identifier and RO-Crate slices build on `PersistentIdentifier`, `PersistentIdentifierParse`, `PID`, `Identifier`, `TypedIdentifier`, `ResearchActivity`, `ResearchOutput`, and the existing crosswalk loss-reporting contract. Adding a general activity `identifiers` collection must preserve the broader RAD field and standards-adapter contract in the [standards field matrix](../assets/standards-field-matrix.md). dPID must reuse that extensible field instead of adding another scheme-specific `meta.dpid` field.

The resolver slice uses ACORN's bounded HTTP transport, configurable endpoints, `ResolutionOutcome`, discovery provenance, and the implemented shared link-evidence contract. It must coordinate `io/api/mod.rs` and `application.json` with the API shared-contract lane, `analyzer/host/discovery/mod.rs` with the analyzer integration owner, `transfer.rs` with the research-activity store lane, and CLI roots/generated help with the CLI composition lane. The portable parser and RO-Crate codec remain usable without network access and must preserve the supported no-default-features/WASM boundary.

No first-release dependency is taken on an IPFS client, Ceramic/Codex SDK, blockchain RPC, wallet, DID method driver, nanopublication library, or mutable dPID registry operation.

# Goal

Add the smallest useful, reproducible dPID lifecycle to ACORN:

1. recognize explicit dPID URIs and known dpid.org resolver URLs without interpreting arbitrary integers as identifiers;
2. distinguish the stable dPID root, a selected immutable version, and an optional component path;
3. resolve a root or version through the public read-only resolver and retain the exact manifest CID, version commit, timestamp, registry identity, and owner evidence returned;
4. decode the resolver's RO-Crate 1.1 JSON-LD representation into a portable typed subset;
5. crosswalk research-object metadata and listed components into RAD and `ResearchOutput` without manufacturing authorship, activity dates, or identity claims;
6. expose the same behavior through `acorn gather --resolve` and an explicit `acorn import dpid` workflow; and
7. fail closed and retain locally valid evidence when a resolver response is unavailable, malformed, internally inconsistent, or outside the supported profile.

The initial feature is intentionally read-only. It does not mint or update dPIDs, sign transactions, manage wallets or keys, pin or download IPFS content, or claim cryptographic verification of resolver-provided CIDs.

# Current evidence

## ACORN

- `crates/acorn-schema/src/pid/mod.rs` has a closed `PID` enum and central normalization, identity, discovery, display, project-identifier, and resolver-dispatch behavior. It has no dPID variant or typed root/version/path representation.
- `ResearchActivityMetadata` has a local kebab identifier and several scheme-specific PID fields but no extensible primary activity-identifier collection. `TypedIdentifier` already represents arbitrary schemes and scheme URIs for related schema positions, so a general `identifiers` field is the compatible direction.
- `ResearchOutput` can preserve primary, alternate, and related identifiers plus version, rights, formats, and descriptive metadata. It is the correct RAD projection for RO-Crate components, subject to explicit loss reporting.
- `ArtifactCandidate::identity_keys` and `to_partial_rad_json` currently use fixed project-PID mappings. `transfer.rs` likewise derives canonical database keys from a fixed DOI/Handle/ISBN/Patent/RAiD/SWHID list. Both need the same root-level dPID identity rule to avoid version- or path-specific duplicate projects.
- Discovery resolution currently has typed outcomes for selected PIDs and can preserve provider metadata, but neither `Record::resolve` nor resolved candidate assembly understands dPID history or RO-Crate.
- `io/api/mod.rs` and `application.json` provide the existing configurable provider boundary. Endpoint snapshot/count tests currently assume 18 entries, so dPID registration is an explicit integration lock.
- `acorn import` currently supports only model and specification subcommands. A dPID import leaf can own transformation and output while shared CLI parsing and generated help remain serialized integration points.
- There is no RO-Crate schema module and no dPID fixture family in the repository.

## Upstream contracts

- dPID describes a persistent identifier that maps a stable numeric root and its versions to content-addressed manifests. The project remains under active development, so ACORN must pin fixtures and treat the resolver wire format as a versioned provider contract rather than a permanent standard ([dPID project](https://www.dpid.org/)).
- The current open-source resolver exposes a v2 API over legacy and stream-backed registry data. Its documented route accepts `/v2/resolve/dpid/{dpid}/{version?}`; its universal route supports `raw`, `json`, `myst`, and `jsonld` formats. History responses include a registry id, owner, current manifest, and ordered version records ([resolver source at pinned commit](https://github.com/desci-labs/dpid-resolver/tree/7dcd59b9aeb0ac70f80e3c7bf4df394477890f75), [v2 resolve route](https://github.com/desci-labs/dpid-resolver/blob/7dcd59b9aeb0ac70f80e3c7bf4df394477890f75/src/api/v2/resolve.ts)).
- The resolver validates dPID roots as decimal digits and accepts either `vN` or a numeric API version index. ACORN should emit only the human-facing `vN` form and should not expose the ambiguous zero-based numeric API form as identifier syntax ([validation source](https://github.com/desci-labs/dpid-resolver/blob/7dcd59b9aeb0ac70f80e3c7bf4df394477890f75/src/util/validation.ts)).
- Live probes on 2026-09-08 confirmed that both legacy and stream-backed records return manifest/version history and that public `?format=jsonld` resolution emits an RO-Crate-like JSON-LD graph. The samples also contained missing/uneven optional values, so decoding must be tolerant of optional provider fields while rejecting missing identity or manifest invariants.
- RO-Crate 1.1 represents a research object as JSON-LD. The root links data entities through `hasPart`; files use `File`, directories use `Dataset`, and contextual entities describe people, organizations, publications, and other metadata ([RO-Crate 1.1 data entities](https://www.researchobject.org/ro-crate/specification/1.1/data-entities.html)). The observed dPID profile sometimes types the root as `CreativeWork`, so ACORN needs a documented resolver-profile adapter rather than pretending the upstream output is a fully conformant generic crate.

# Concept boundaries: dPID, nanopublications, and DIDs

These technologies can occur in the same scholarly graph, but they identify different things and imply different verification work.

| Concept | Primary subject | Resolution result | Mutability/version model | ACORN treatment in this plan |
|---|---|---|---|---|
| dPID | A versioned research object and its component paths | Manifest history or a selected representation such as RO-Crate JSON-LD | Stable root selects evolving history; explicit version selects one manifest | First-class project identifier, read-only resolution, RO-Crate import, exact manifest/version provenance |
| Nanopublication | One atomic RDF assertion with provenance and publication information | RDF graphs for assertion, provenance, and publication info | Trusty URI identifies immutable content; updates normally publish and link another nanopublication | Out of scope; preserve a nanopublication URI as a typed related identifier when encountered |
| W3C DID | A controller/subject in a DID method | DID Document containing verification methods, controllers, and services | Method-specific create/read/update/deactivate operations | Out of scope; preserve an owner string as opaque provenance even if it is a DID |

A dPID is not a W3C DID: it does not use the `did:<method>:<method-specific-id>` syntax and its resolver returns research-object state rather than a DID Document. A dPID version is also not a nanopublication: the manifest packages a research object, whereas a nanopublication packages an atomic RDF statement and its provenance. Generic DID resolution, signature verification, controller authorization, nanopublication RDF validation, trusty-URI verification, publication, and federation require separate plans ([W3C DID Core](https://www.w3.org/TR/did/), [nanopublication guidelines](https://nanopub.net/guidelines/working_draft/)).

# Support contract

| Capability | Required dPID behavior |
|---|---|
| Find | Discover explicit `dpid://...` values and supported `https://dpid.org/...` host forms. Never infer dPID from an unlabeled integer. |
| Parse | Represent an ASCII-decimal root as a string, an optional explicit `vN` version, and an optional percent-decoded-safe component path. Do not convert the root to a machine integer. |
| Validate | Reject empty/zero versions, signs, whitespace, query/fragment identity, userinfo, lookalike hosts, empty path components, decoded dot segments, encoded separators, and inputs beyond documented length budgets. |
| Normalize | Emit `https://dpid.org/<root>[/vN][/component/path]`; accept approved legacy/public hosts as input but never preserve them as canonical identity. |
| Identity | Use `dpid:<root>` as the project merge key. Preserve normalized root/version/path and resolved manifest separately; never merge projects by owner, stream id, CID, or component path. |
| Resolve | Resolve a root to latest history or an explicit `vN` to one selected manifest through a configurable HTTPS v2 endpoint. Never send a component path before root/version resolution succeeds. |
| Pin | Record the resolver-selected version and manifest CID even when the user supplied only a root, so an import can be reproduced. Do not describe an unversioned root lookup as immutable. |
| Decode | Accept the pinned dPID resolver's narrow history and RO-Crate JSON-LD profiles with bounded bodies, typed required invariants, optional unknown fields, and actionable parse errors. |
| Crosswalk | Map root research-object metadata to RAD and `hasPart` entities to outputs; report every unsupported or ambiguous projection through the existing loss contract. |
| Provenance | Retain endpoint, normalized source dPID, registry id, opaque owner, selected version commit/time, manifest CID, retrieval time, and profile/format. Registry version time is not a RAD activity date. |
| Offline/failure | Make no dPID request in portable/offline paths. `gather --resolve --offline` and networked import fail before requesting; local parsing and validation remain available. |

# Recommendation

## Identifier and identity model

Add a portable `DPID` value object rather than storing an unparsed URL. Preserve the root as its canonical decimal string so ACORN does not inherit JavaScript number precision or commit to an undocumented registry integer width. Normalize leading zero policy from pinned upstream fixtures before implementation; until that evidence exists, parsing must preserve the lexical root and identity comparison must not silently rewrite it.

Use a canonical HTTPS display because that is directly actionable in existing ACORN output, while accepting `dpid://` as an explicit input form. A version is expressed only as `vN` in ACORN-facing identifiers. Numeric version indices remain an internal provider detail if required by an upstream compatibility fixture. Component paths are locators within a selected research object, not distinct project identities.

Add `ResearchActivityMetadata.identifiers: Option<Vec<TypedIdentifier>>` with compatibility-preserving omission when empty. Store the root or versioned dPID there with scheme `dPID` and canonical scheme/resolver URI. Keep the local kebab `identifier` unchanged and do not add a dedicated `dpid` field. Discovery and database identity use the root key; imports also preserve the exact source dPID, selected version, and manifest in typed provenance so reproducibility does not depend on the latest pointer.

## Resolver and trust boundary

Implement one read-only provider against the pinned v2 shape. Resolve history first, select the requested or current version deterministically, verify that the selected manifest occurs in the returned history, then request JSON-LD for that exact version. Treat owner and registry ids as opaque strings. An owner resembling a wallet address or DID is neither an author nor proof of control, and it must never populate contributors or organizations without explicit crate metadata.

The resolver response is evidence from a gateway, not proof that downloaded bytes match a CID. The first release therefore imports metadata only. It may retain IPFS/CID locators as identifiers, but it does not fetch files, traverse arbitrary gateway trees, verify multihashes, materialize component paths, or claim end-to-end integrity. A later materialization plan must reuse ACORN's safe-path/archive boundary and add CID verification before writing content.

## RO-Crate profile and RAD crosswalk

Create a narrow portable RO-Crate 1.1 model around JSON-LD graph ids/types and the properties observed or required for dPID import. Support arrays or single values only where JSON-LD permits and fixtures prove the representation. Locate the root by `@id: "./"`; accept the observed `CreativeWork` root as a documented dPID extension while also supporting the standard `Dataset` root. Resolve `hasPart` references within the graph without network access.

Project root metadata as follows:

| RO-Crate/dPID source | RAD destination | Rule |
|---|---|---|
| normalized root/version dPID | `meta.identifiers` | Always retain; versioned source remains exact while root provides the merge key |
| root `name`, `description` | RAD title/description fields | Map only non-empty scalar or language-selected values supported by the typed profile |
| root `creator`/`author` references | contributors/organizations | Follow local graph references; never use resolver `owner` as a creator |
| root `license` | rights/license representation | Preserve URI and label where available; warn when the RAD vocabulary cannot express the source exactly |
| root keywords/subjects | subjects/keywords | Deduplicate deterministically without inventing vocabulary URIs |
| root `hasPart` graph entities | `ResearchOutput` | Create one output per local file/dataset/creative-work entity; preserve graph order for stable diagnostics |
| entity `@id`, CID/IPFS locator | output primary/alternate identifiers | Prefer the stable graph id as local relation and retain CID/IPFS values as typed alternates |
| entity `name`, `description`, `encodingFormat`, `version`, `license` | output metadata | Map directly when type-compatible; otherwise emit a loss warning |
| resolver version time | import provenance only | Never infer started, completed, published, or modified activity dates |
| unknown graph entity/property | retained source plus loss report | Never silently drop an unknown value while claiming lossless import |

The crosswalk should return a value plus structured warnings, following the implemented RAD crosswalk contract. Missing optional descriptions or labels do not fail import; missing root identity, selected manifest, graph root, or an internally referenced required entity does. Duplicate graph ids and conflicting scalar values fail closed rather than using last-write-wins behavior.

## User workflows

`acorn gather --resolve` should recognize dPIDs and record a normal resolved/failed lifecycle outcome. A root dPID is a project identifier, while a component-path dPID contributes evidence to the root candidate and may become an output during successful RO-Crate assembly. Duplicate roots should produce one logical history lookup and one exact JSON-LD lookup per selected version.

Add `acorn import dpid <DPID> [OUTPUT]` as the explicit transformation workflow. A root input resolves latest and pins the returned version/manifest in output provenance; a versioned input imports that exact selection. A component path is rejected for whole-project import in the first release. Follow existing import output/overwrite conventions rather than creating a dPID-specific filesystem policy. Import does not download component bytes.

# Implementation plan

## 0. Freeze the resolver/profile contract and fixtures

**Atomic write set:** `tests/fixtures/dpid/`.

- Capture redacted, attribution-documented fixtures from the pinned resolver commit/profile for one legacy record and one stream-backed record: history, exact-version RO-Crate JSON-LD, not found, malformed payload, missing manifest, conflicting version, and duplicate graph-id cases.
- Record the source URL, retrieval date, response media type, and expected root/version/manifest relationship beside each fixture without embedding mutable access credentials.
- Decide and test leading-zero behavior, maximum accepted root/version/path lengths, version numbering, supported public host aliases, JSON-LD scalar/array forms, and the observed root types before public Rust types are implemented.
- Keep live probes manual. Required tests must be deterministic and make no request to dpid.org, IPFS, Ceramic/Codex, or a blockchain node.

**Gate:** every later acceptance criterion can be demonstrated by a named fixture, and unresolved upstream ambiguity is recorded rather than guessed in code.

## 1. Add a portable dPID parser and PID wiring

**Atomic write set:** `crates/acorn-schema/src/pid/dpid/{mod.rs,tests.rs}`, `crates/acorn-schema/src/pid/{mod.rs,tests.rs}`.

- Implement immutable root/version/path components, validation, canonical HTTPS display, `dpid://` and known-host parsing, URL extraction, schema URI, and `dpid:<root>` identity.
- Add `PID::DPID` to normalization, display, generic discovery, resolver eligibility, and project-identifier classification. Discovery requires an explicit scheme/host; typed parsing may accept only the forms frozen in Slice 0.
- Keep parsing allocation-only and independent of `std`, HTTP, filesystem paths, and JSON-LD. Do not use a numeric Rust type for the root.
- Add property-style and table tests for canonical round trips, human/version/component forms, Unicode digits, leading zeros, invalid versions, percent encoding, traversal, query/fragment, deceptive hosts, surrounding punctuation, duplicates, and identity equality across root/version/path.

**Gate:** `cargo nextest run -p acorn-schema --lib pid::dpid`, generic PID tests, no-default-features, alloc, and WASM checks pass.

## 2. Add extensible activity identifiers and root persistence

**Atomic write set:** `crates/acorn-schema/src/research_activity/{mod.rs,common.rs}`, adjacent research-activity tests, `crates/acorn-lib/src/io/config/transfer.rs`.

- Add the optional `identifiers` collection to activity metadata and one pure scheme-aware coherence validator for dPID values and scheme URI. Preserve unknown explicitly named schemes and existing serialized RAD documents.
- Define deterministic deduplication and ordering by normalized scheme plus value. Multiple versions of one dPID may coexist as provenance, but candidate/database identity collapses only on `dpid:<root>`.
- Teach project database key derivation to consume general activity identifiers and emit the dPID root key without replacing current scheme-specific keys or the local RAD key.
- Add round-trip, schema-generation, invalid-scheme, duplicate, multiple-version, and migration-compatibility tests. Serialize any shared `TypedIdentifier` validator change through the schema-features lane.

**Gate:** focused research-activity and transfer tests prove old documents retain their shape and all dPID versions map to the same project identity.

## 3. Implement the portable dPID RO-Crate profile and crosswalk

**Atomic write set:** `crates/acorn-schema/src/standard/rocrate/mod.rs`, `crates/acorn-schema/src/standard/{mod.rs,tests/mod.rs}`, `crates/acorn-schema/src/standard/tests/rocrate.rs`, `crates/acorn-schema/src/research_activity/output.rs` only if the frozen mapping exposes a missing reusable field.

- Decode the narrow JSON-LD graph/profile described above with `serde`, explicit required invariants, deterministic graph indexing, and unknown-field preservation or loss reporting.
- Resolve only local graph references. Never dereference external JSON-LD contexts, remote entity ids, IPFS links, or DID service endpoints in the portable layer.
- Implement a pure `RO-Crate -> Result<(ResearchActivity, Vec<ConversionWarning>)>` transformation through the existing crosswalk trait. Keep root metadata, outputs, people/organizations, rights, identifiers, and warnings independently testable.
- Add legacy and stream-backed snapshots plus malformed graph tests, scalar/array representation tests, duplicate-id/conflict tests, stable output-order tests, and a re-import idempotence test where the RAD model permits it.

**Gate:** `cargo nextest run -p acorn-schema --lib standard::tests::rocrate` and the full portable feature matrix pass.

## 4. Add the read-only dPID resolver provider

**Atomic write set:** `crates/acorn-lib/src/io/api/dpid.rs`, `crates/acorn-lib/src/io/api/tests/dpid.rs`.

- Define narrow typed history DTOs and a resolved selection containing source dPID, registry id, opaque owner, selected version/commit/time, manifest CID, and decoded RO-Crate.
- Implement fixture-injectable history and exact-version JSON-LD calls through existing bounded transport. Build route components only from validated `DPID` values, require HTTPS outside the in-process test server, and set explicit response-size/time limits.
- Select latest deterministically from returned history rather than trusting array position alone. Reject missing, duplicate, or conflicting requested versions/manifests and a JSON-LD response inconsistent with the selection.
- Separate provider availability, metadata consistency, and cryptographic verification in statuses/messages. No successful resolver response may be labeled CID-verified.
- Cover legacy/stream-backed success, versioned/root selection, not found, unsupported version, timeout, HTTP error, content type, oversize body, malformed JSON, inconsistent history, invalid input before network, and opaque DID/wallet-like owners with local fixtures/server tests.

**Gate:** `cargo nextest run -p acorn-lib --lib io::api::tests::dpid` passes without public network access.

## 5. Integrate dPID resolution and candidate assembly into gather

**Atomic write set:** `crates/acorn-lib/src/io/api/{mod.rs,tests/mod.rs}`, `crates/acorn-lib/assets/constants/application.json`, `crates/acorn-lib/src/analyzer/host/discovery/{mod.rs,tests/mod.rs}`.

- Register a configurable dPID v2 endpoint and update endpoint snapshots/count assertions in one integration commit.
- Dispatch root/version records through the dPID provider, serialize the typed outcome through existing discovery provenance, and render the canonical versioned dPID plus manifest summary without implying verification.
- Group duplicate root/version requests, assemble a root project candidate from the crosswalk, and attach component-path evidence to the root. Use the same root identity contract as transfer/database persistence.
- Preserve local identifiers and structured failures for not found, malformed, unavailable, and inconsistent responses. Non-`--resolve` and offline paths perform no request.
- Test lifecycle statuses, root/version deduplication, component evidence, latest pinning, provenance serialization, candidate merge behavior, no owner-to-contributor inference, no activity-date inference, and offline behavior.

**Gate:** focused API/discovery tests and `cargo nextest run -p acorn-cli commands::gather` pass.

## 6. Add explicit dPID import and documentation

**Atomic write set:** `crates/acorn-cli/src/commands/import/dpid.rs`, `crates/acorn-cli/src/commands/{mod.rs,import/mod.rs,import/tests/mod.rs}`, `crates/acorn-cli/src/cli/{mod.rs,arguments/mod.rs,tests/mod.rs}`, `docs/src/concepts/persistent-identifiers.md`, `docs/src/cli-reference/commands/{gather,import}.md`, and generated command help.

- Add the typed `import dpid` arguments and compose the provider/crosswalk without duplicating parsing, resolution, or output-write logic.
- Require network mode, reject a component path, honor existing output collision/formatting conventions, and display all crosswalk warnings before reporting success.
- Document root versus version identity, mutable-latest behavior, retained manifest provenance, metadata-only scope, endpoint trust, offline behavior, and recovery from failed imports.
- Add the dPID/nanopublication/DID distinction to PID documentation so users do not infer DID controller verification or nanopublication semantics from dPID support.
- Regenerate command help and test root/version imports, invalid identifiers, component rejection, output collisions, warning rendering, resolver failure, and no partial output after failure.

**Gate:** focused import/CLI parse tests and `make doc` pass.

# Risks and mitigations

| Risk | Impact | Mitigation |
|---|---|---|
| The actively developed resolver changes routes or response fields. | High | Pin source/fixtures, isolate DTOs in one provider leaf, configure the endpoint, tolerate only optional additive fields, and fail closed on identity/manifest drift. |
| An unversioned root changes between discovery and import. | High | Resolve history and the exact JSON-LD selection as one bounded workflow, persist the selected version/manifest, and never call latest output immutable. |
| Resolver availability is mistaken for content integrity. | High | Label gateway evidence and CID verification separately; make no integrity claim until a later downloader recomputes the content identifier. |
| RO-Crate output is incomplete, inconsistent, or outside the observed profile. | Medium | Validate graph invariants, preserve unknown values or emit structured losses, and use legacy plus stream-backed golden fixtures. |
| Registry owner metadata is mistaken for scholarly authorship or DID control. | High | Keep owner opaque in provenance and populate contributors only from explicit RO-Crate graph relationships. |
| Root/version/path forms split or over-merge canonical projects. | High | Centralize one typed parser and root identity key, then test that discovery, RAD, transfer, and database consumers agree. |
| Component paths become a traversal or unsafe-write vector. | Medium | Validate them as URI components, perform no first-release materialization, and require a later downloader to reuse `SafePath`. |
| The general RAD identifier field conflicts with ongoing standards work. | Medium | Land it under the schema-features lock, reuse `TypedIdentifier`, preserve omission/backward compatibility, and require RAD crosswalk review. |

# Validation

Run each slice's focused gate, followed by:

```sh
cargo fmt --all --check
cargo nextest run -p acorn-schema --lib pid::dpid
cargo nextest run -p acorn-schema --lib research_activity
cargo nextest run -p acorn-schema --lib standard::tests::rocrate
cargo nextest run -p acorn-lib --lib io::api::tests::dpid
cargo nextest run -p acorn-lib --lib analyzer::host::discovery
cargo nextest run -p acorn-cli commands::gather
cargo nextest run -p acorn-cli commands::import
cargo check -p acorn-schema --no-default-features
cargo check -p acorn-schema --no-default-features --features alloc
make check-wasm
make lint
make test
make doc
```

No required test contacts a public dPID resolver, IPFS gateway, Ceramic/Codex node, DID resolver, or nanopublication service. If implementation adds a crate dependency, run `make audit` and document why the existing URL/Serde/HTTP stack was insufficient before changing manifests.

# Acceptance criteria

1. Explicit `dpid://` and approved dpid.org HTTPS forms round-trip to one canonical HTTPS dPID; arbitrary integers, lookalike hosts, unsafe paths, and ambiguous numeric API-version forms are not discovered.
2. Root, `vN`, and component path remain distinct typed components, while every version/path of one root yields exactly the same `dpid:<root>` project identity key.
3. dPID roots are never converted to machine integers, and the fixture-approved leading-zero and length rules are enforced consistently in parse, display, identity, and request construction.
4. Existing RAD documents deserialize and reserialize compatibly; a valid dPID is retained in the general activity identifiers collection with coherent scheme metadata and contributes the same root key to discovery and database persistence.
5. Legacy and stream-backed fixtures both resolve to a typed, exact selection containing registry id, opaque owner, selected version/commit/time, manifest CID, and RO-Crate metadata.
6. A root/latest import records the exact resolved version and manifest; a versioned import selects only that version; an unsupported or inconsistent version fails without partial output.
7. The RO-Crate adapter locates one root, resolves local `hasPart` references, maps supported root/output/person/organization/rights fields, and reports unsupported or ambiguous values through the crosswalk loss contract.
8. Resolver owner values never become contributors, organizations, rights holders, DID controllers, or proof of authorship; registry timestamps never become RAD activity or publication dates.
9. `acorn gather --resolve` performs at most one logical history lookup per dPID root and one JSON-LD lookup per selected version, produces project candidates by root, retains component evidence, and preserves actionable failed outcomes.
10. `acorn import dpid` imports metadata only, rejects component-path project imports, honors normal output safety, surfaces loss warnings, and never downloads or materializes referenced files.
11. Documentation states that dPID is neither a W3C DID nor a nanopublication and makes no claim of controller, signature, trusty-URI, CID, or file-integrity verification.
12. Portable schema/WASM checks, focused tests, the full test suite, lint, generated help, and documentation pass with no live-network dependency.
