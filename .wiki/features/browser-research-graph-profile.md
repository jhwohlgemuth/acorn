# RAD graph projection and Turtle profile

This is the GPH-01 contract for the [browser graph plan](browser-research-graph.md). It describes a read-only projection of validated RAD records. Every input has a caller-supplied source key and normalized relative path. The source key distinguishes a local workspace from each remote bucket and revision. Replacing a bucket creates a new snapshot; no graph identity survives a changed source key by accident.

## Identity and provenance

- A node ID is `urn:acorn:graph:node:<kind>:<hex-encoded-UTF-8-key>`. The kind separates otherwise equal keys. Hex encoding is lowercase, injective, and creates valid absolute IRIs without guessing URI escaping rules. The exact ID is an opaque UI key, not a claimed public PID.
- An activity key is the source key plus the relative file path. A project key is the source key plus the RAD `meta.identifier`. Project grouping is source-local until the combined-bucket contract proves a stronger identity. A single project may contain multiple RAD file records.
- A structured person or organization is shared across activities only when it has a validated explicit PID with a scheme and normalized value. An entity without one uses the containing activity key plus its field index. Names are labels, never cross-record identity. The explicit PID is its own Identifier node; the person/organization node retains the chosen PID key.
- A related target resolves to a loaded activity only when its identifier unambiguously matches a loaded activity's source-qualified path, project identifier, or normalized explicit PID. Ambiguous or absent matches produce a Reference node with the authored identifier and relation type; they do not invent a RAD record. A reference may be reused by exact scheme/value and source scope. No remote dereference occurs.
- Each edge records its source RAD path and field path. Its stable key is source node ID, edge kind, target node ID, source key/path, and field path. Duplicate authored occurrences may be coalesced only when those values agree. Nodes and edges sort by stable ID before exposure.

## Node and edge types

| Node | Label and evidence |
| --- | --- |
| Activity | RAD `title`; source key and file path |
| Project | RAD `meta.identifier`; a representative activity title may be shown but does not establish identity |
| Organization | `meta.organizations[i].name`, with optional validated identifier |
| Contributor | `meta.contributors[i].name`, with optional validated identifier |
| Identifier | Explicit typed PID value and scheme; may be attached to an activity or structured entity |
| Reference | Unresolved or ambiguous `meta.relatedResources[i].identifier`; visibly marked as external/unresolved |

| Directed edge | Source field | Meaning |
| --- | --- | --- |
| Project → Activity: contains | `meta.identifier` | This RAD belongs to the source-local project grouping. |
| Activity → Organization: involves | `meta.organizations[i]` | The organization is explicitly listed on this RAD. No collaboration or funding role is inferred. |
| Activity → Contributor: credits | `meta.contributors[i]` | The person is explicitly listed on this RAD. No authorship role is inferred. |
| Activity/Organization/Contributor → Identifier: has identifier | Explicit PID field or structured `identifiers[i]` | Authored identifier, not a resolver result. |
| Activity → Activity/Reference: relates to | `meta.relatedResources[i]` | An authored relation; preserve `relationType` as edge metadata. |

Legacy `related` is an input alias of `relatedResources`. `keywords` are searchable detail values, not graph nodes. Legacy free-text `partners`, `sponsors`, and `contact` do not create entity nodes. The graph includes only records accepted by the existing workspace validation. File paths and source keys are provenance metadata, not names to expose in an automatically shared URL.

## Turtle profile

Export one RDF default graph from the same projected nodes and edges. Every node subject is its stable node URN. Each node has `rdf:type` with an ACORN class URN (`urn:acorn:graph:class:<kind>`) and `rdfs:label` with its display label. Every supported edge becomes one triple using `urn:acorn:graph:predicate:<edge-kind>`; the five predicate local names are `contains`, `involves`, `credits`, `hasIdentifier`, and `relatesTo`. `relatesTo` does not collapse authored `relationType` into a stronger standard predicate. The edge-specific type and field provenance remain in the in-memory projection; Turtle v1 intentionally omits them rather than asserting unsupported RDF reification. Source file path, source key, graph layout, search/filter state, keywords, and local-only unnamed agents are not exported as literal provenance. A node with no exportable edge still emits type and label triples. All literals use Turtle escaping; no untrusted string is inserted as raw syntax. Order triples lexically for deterministic bytes. This is an ACORN-defined interoperable RDF profile, not a complete RAD serialization or an RDF import format.

The vocabulary uses standard [RDF type](https://www.w3.org/TR/rdf11-concepts/#dfn-type) and [RDFS label](https://www.w3.org/TR/rdf-schema/#ch_label). The domain-specific predicates are ACORN URNs because [DCMI terms](https://www.dublincore.org/specifications/dublin-core/dcmi-terms/) and [PROV-O](https://www.w3.org/TR/prov-o/) would add semantics such as authorship, part/whole, or derivation that a generic RAD membership does not necessarily establish.

## Sample expectation

Two validated RAD files from source `local`, paths `a.json` and `b.json`, both with `meta.identifier: "project-one"`, produce one Project node, two Activity nodes, and two `contains` edges. If both list organization `Example Lab` without a structured identifier, the graph has two Organization nodes. If both list the same valid ROR identifier, they share one Organization node and one Identifier node. An explicit related reference from `a.json` to `b.json` produces one directed `relatesTo` edge when the key resolves unambiguously; otherwise it ends at a Reference node. Reordering input records does not change node IDs, edge IDs, or Turtle bytes.

## Verification record

- 2026-11-25 source review: `crates/acorn-schema/src/research_activity/mod.rs:126-171,216-266,367-425` confirms typed RAD, project identifier, contributors, organizations, and related resources with the legacy alias. `common.rs:80-120,273-315` confirms structured entity names/identifiers and related identifier/type. `crates/acorn-web/src/research_index.rs:72-150,186-245` confirms current validated-workspace grouping and reference matching. The existing schema test module contains typed contributor and identifier validation cases.
- Focused RAD fixture/codec command and outcome: `cargo nextest run --package acorn-schema research_activity` passed 148 tests on 2026-11-25 before the graph module was added. After implementation, `cargo nextest run --package acorn-schema research_activity::graph` passed eight graph/Turtle tests, `cargo clippy --package acorn-schema --all-targets --no-deps -- -D warnings` and `cargo fmt --package acorn-schema --check` passed, and the portable alloc build passed with `RUSTC=/Users/o9w/.rustup/toolchains/stable-aarch64-apple-darwin/bin/rustc cargo check -p acorn-schema --target wasm32-unknown-unknown --no-default-features --features alloc`. The explicit `RUSTC` is needed because Homebrew `cargo` otherwise selects Homebrew `rustc`, which lacks this WASM sysroot.
