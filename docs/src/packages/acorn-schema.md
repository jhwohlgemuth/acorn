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

Local PID validation checks parsed structure and, for ISBNs, ISNIs, ORCIDs, and RORs, the implemented check digit. It does not confirm registration with an external authority.
