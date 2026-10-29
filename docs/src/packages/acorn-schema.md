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

Local PID validation checks parsed structure and, for ISBNs, ISNIs, ORCIDs, and RORs, the implemented check digit. It does not confirm registration with an external authority.

## Computation complexity

Use `bigo!` to check a Big-O string literal during compilation:

```rust
use acorn_schema::{bigo, computation::ComplexityBound};

let bound = ComplexityBound::Time(bigo!("O(n log n)").into());
```

The same grammar validates complexity strings in research activity JSON. For example, `{"case":"worst","bounds":[{"time":"O(n log n)"}],"variables":{"n":"number of records"}}` describes a computation bound. Validation checks notation and, when a variables map is supplied, its referenced names; it does not prove the claimed complexity or infer it from measurements.
