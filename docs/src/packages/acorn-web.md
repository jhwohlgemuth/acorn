# WebAssembly development

ACORN's portable Rust surface can be checked for the `wasm32-unknown-unknown` target. This is currently a portability build for contributors, not a published JavaScript or browser package.

> [!IMPORTANT]
> `make wasm` verifies the portable and analyzer-only feature paths. It does not link a `.wasm` artifact or generate JavaScript bindings.

## Installation

Install the Rust toolchain and add the WASM target:

```sh
rustup target add wasm32-unknown-unknown
```

Clone the ACORN repository, then run the check from its root:

```sh
make wasm
```

Maintainers can check the complete native and WASM portability matrix plus the host-dependency deny policy with:

```sh
make check-portable
```

The equivalent Cargo command is:

```sh
cargo check -p acorn-lib --no-default-features --features wasm --target wasm32-unknown-unknown
cargo check -p acorn-lib --no-default-features --features analyzer --target wasm32-unknown-unknown
```

Rust projects that compile to WASM can select the same ACORN surface:

```sh
cargo add acorn-lib --no-default-features --features wasm
```

## Currently available functionality

The WASM build exposes Rust APIs that operate on supplied in-memory values. It currently includes:

| Capability | Available functionality |
|---|---|
| Research metadata | `ResearchActivity` and its supporting metadata, section, linked-data, and media types |
| Metadata standards | Serde-compatible CFF, DataCite, DCAT, Dublin Core, Huwise, Invenio, and text models; in-memory schema crosswalk contracts and conversions |
| Persistent identifiers | ARK, DOI, ISBN, ISNI, ORCID, RAiD, ROR, and patent parsing, recognition, normalization, component access, and checksum validation where the identifier defines one |
| Validation | Research schema validation plus helpers for identifiers, dates and timestamps, URLs, semantic versions, geographic coordinates and polygons, language and country codes, phone numbers, and media extensions |
| Hardware metadata | Architecture, accelerator, processor, memory, sensor, topology, vendor, and computing-resource models |
| Portable domain APIs | `Location`, `Repository`, `Scheme`, `Release`, `AcornError`, and `AcornResult` |
| Portable utilities | MIME and file-extension detection, Crockford Base32, constant-time byte comparison, glob matching, regex-to-glob conversion, frontmatter splitting, semantic versions, byte formatting, and string normalization |
| Reference data | Compile-time vocabularies used by portable schema and validation code |
| Analyzer kernel | Check, error, Vale output, readability, and scoring models plus calculations over supplied text through the separate `analyzer` feature |

Schema types implement their normal Serde, JSON Schema, and validation traits in this configuration. Callers provide the strings, bytes, or already-deserialized values; ACORN does not fetch or read them in the WASM build.

For example, a Rust component compiled to WASM can use ACORN's identifier validation directly:

```rust
use acorn_schema::schema::validate::{is_arxiv, is_doi, is_isni, is_orcid, is_ror};

assert!(is_doi("10.11578/dc.20250604.1").is_ok());
assert!(is_arxiv("arXiv:2106.09685").is_ok());
assert!(is_isni("https://isni.org/isni/0000000492299539").is_ok());
assert!(is_orcid("https://orcid.org/0000-0002-2057-9115").is_ok());
assert!(is_ror("https://ror.org/01qz5mb56").is_ok());
```

## What the check covers

The current `wasm` feature selects allocation-capable APIs without enabling ACORN's native filesystem, process, terminal presentation, database, or network services. The separate `analyzer` feature adds only the portable analyzer kernel. CI compiles both paths and rejects host-only dependencies from the analyzer graph.

The check does not currently provide:

- `wasm-bindgen` JavaScript bindings
- an npm package
- browser HTTP or storage adapters
- a supported browser application entry point
- native file or URL reads and writes
- analyzer host execution, document conversion, database, process, download, or doctor services
- std-only agent definitions and embedded agent assets

Those capabilities are planned for a dedicated `acorn-wasm` adapter crate. Portable crates will remain independent of browser-specific dependencies, and `acorn-wasm` will compose supported ACORN services with browser implementations.

## macOS sandbox warnings

Restricted macOS environments may report that `xcrun` could not create a cache file while locating `MacOSX.sdk`. This is a host toolchain warning rather than a WASM compilation failure. If Cargo prints `Finished` and the command exits successfully, the portability check passed.

## Maintainer references

The public guide documents supported developer behavior. Implementation status and architectural decisions are tracked in the internal [WASM feature path](https://code.ornl.gov/research-enablement/acorn/-/blob/main/.wiki/refactors/portability/wasm-feature-path.md) and [no-std/WASM CI plan](https://code.ornl.gov/research-enablement/acorn/-/blob/main/.wiki/refactors/portability/no-std-wasm-ci.md).
