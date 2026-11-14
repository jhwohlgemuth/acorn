# Rust schemas

[`acorn-schema`](https://crates.io/crates/acorn-schema) contains ACORN's portable data model, validation rules, persistent identifiers, and metadata-standard types. It does not require ACORN's host filesystem, process, database, or network services.

## Installation

Add the crate to a Rust project:

```sh
cargo add acorn-schema
```

Schema and validation functions can then be used directly:

```rust
use acorn_schema::validation::rules::{ark, arxiv, doi, isni, orcid, ror};

assert!(ark("ark:/1234/w5678").is_ok());
assert!(arxiv("arXiv:2106.09685").is_ok());
assert!(doi("10.11578/dc.20250604.1").is_ok());
assert!(isni("https://isni.org/isni/0000000492299539").is_ok());
assert!(orcid("https://orcid.org/0000-0002-2057-9115").is_ok());
assert!(ror("01qz5mb56").is_ok());
```

See the [`acorn-schema` API documentation](https://docs.rs/acorn-schema/latest/acorn_schema/) for the complete public surface and [Persistent identifier lifecycle](../concepts/persistent-identifiers.md) for identifier behavior.

The alloc-capable `acorn_schema::standard::zon` module encodes and decodes the JSON data model without host I/O or an additional runtime dependency. See [ZON serialization](../concepts/zon.md) for the supported profile and file conventions.

The portable `acorn_schema::standard::dublin_core` module models the current DCMI terms, legacy element namespace, DCMI type vocabulary, and DCAM terms as a normalized RDF graph. Its local codecs cover native graph JSON and YAML, JSON-LD, RDF/XML, and Turtle. See [Dublin Core](../concepts/dublin-core.md) for the data model, safety boundaries, and crosswalk behavior.

Research activity records can carry time-coded developments and project-health assessments in an optional logbook. See the [Logbook schema](logbook-schema.md) for the entry variants, validation rules, reporting-window behavior, and data flow into reports and canonical metadata.

Local PID validation checks parsed structure and, for ISBNs, ISNIs, ORCIDs, and RORs, the implemented check digit. It does not confirm registration with an external authority.

RAD typed identifiers dispatch to these validators using their declared scheme: ARK, arXiv, DOI, Handle, ISBN, ISNI, ORCID, patent, RAiD, ROR, RRID, and SWHID. URL identifiers require an absolute HTTP or HTTPS address. Recognized registry URIs can select a scheme when its name is absent; conflicting names and recognized registry URIs are rejected. Unspecified local identifiers, custom schemes, and schemes without a dedicated validator retain nonempty-value validation. These rules apply to nested organization identifiers, affiliations, related resources, and output identifiers. Typed funders also check ROR/ISNI values and registry coherence, while DataCite related items validate identifier types with implemented parsers.

RAD validation also checks required contact and prose text, award identifiers, nested ASPECT data, and ISO 639-3 language codes against an embedded snapshot of the [ISO 639-3 registry](https://iso639-3.sil.org/). Web locations require HTTP or HTTPS with a nonempty host. Role end dates cannot precede their start dates; geographic boxes require south latitude to be at or below north latitude, while longitude bounds may cross the antimeridian. Environment and computation identifiers must be unique within a record, platform identifiers within an environment, and logbook entries must reference their containing activity.

Validation paths use serialized field names, such as `contact.givenName` and `meta.relatedResources`. JSON decoding collects independent errors without treating omitted optional fields as missing. Prose limits count Unicode characters. RAD YAML output uses ordinary mappings for enum values, including hardware resources, so encoded records can be read back without custom YAML tags.

## Computation complexity

Use `bigo!` to check a Big-O string literal during compilation:

```rust
use acorn_schema::{bigo, computation::ComplexityBound};

let bound = ComplexityBound::Time(bigo!("O(n log n)").into());
```

The same grammar validates complexity strings in research activity JSON. For example, `{"case":"worst","bounds":[{"time":"O(n log n)"}],"variables":{"n":"number of records"}}` describes a computation bound. Validation checks notation and, when a variables map is supplied, its referenced names; it does not prove the claimed complexity or infer it from measurements.

The `#[validate(bigo)]` scalar rule uses the same parser for schema fields. Canonical notation supports explicit products, division, rational powers such as `P^(1/2)`, `polylog(P)`, `W_out(P)`, and named-base logarithms such as `log_a(P)`. A complete RAD document reports malformed, undeclared, empty, and repeated time or space bounds at their nested metadata paths with `syntax`, `reference`, `nonempty`, and `duplicate` codes. Bound JSON Schema limits each expression to 256 characters; runtime parsing and variable checks remain authoritative.

For pairwise analysis, `acorn_core::big_o::compare_notation` accepts `ComparisonInput { left, right, assumptions }` and returns `ComparisonOutput { left, right, dominance, assumptions }`. It infers ordinary symbols as `growing` and named logarithm bases as `fixed-log-base`; supplied assumptions override those defaults. The returned map contains the effective assumptions in lexical order. `left` or `right` means that side grows strictly faster. `equivalent`, `incomparable`, and `undetermined` are valid analytical results. Malformed expressions, more than 64 supplied assumptions, and assumptions for names absent from both expressions are errors. The lower-level `compare_growth` API remains available for already parsed expressions and explicit assumptions.
