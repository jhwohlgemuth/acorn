# 🦀 Rust crate
[<img alt="crates.io" src="https://img.shields.io/crates/v/acorn-lib.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/acorn-lib)
[<img alt="docs.rs" src="https://img.shields.io/docsrs/acorn-lib?style=for-the-badge&color=%2300662C&logo=rust" height="20">](https://docs.rs/acorn-lib)
[<img alt="Size" src="https://img.shields.io/crates/size/acorn-lib?style=for-the-badge" height="20">](https://docs.rs/acorn-lib)
[<img alt="Downloads" src="https://img.shields.io/crates/dr/acorn-lib?style=for-the-badge" height="20">](https://docs.rs/acorn-lib)
[<img alt="Dependency Status" src="https://deps.rs/crate/acorn-lib/latest/status.svg?style=for-the-badge" height="20">](https://deps.rs/crate/acorn-lib/latest)

The ACORN CLI application is built on top of the `acorn-lib` Rust crate. The `acorn-lib` library provides core functionalities for working with ACORN [schemas](../concepts/acorn.md), validating persistent identifiers[^pid], and generating artifacts.

## Installation
- Add `acorn-lib` as a dependency in your `Cargo.toml`[^version]
  ```toml
  [dependencies]
  acorn-lib = "0.1.52"
  ```  
- Use `acorn-lib` functions in your Rust code
  ```rust
  use acorn::schema::validate::{is_ark, is_doi, is_orcid, is_ror};

  assert!(is_ark("ark:/1234/w5678"));
  assert!(is_doi("10.11578/dc.20250604.1"));
  assert!(is_orcid("https://orcid.org/0000-0002-2057-9115"));
  assert!(is_ror("01qz5mb56"));
  ```
- If you want to use the full capability of `acorn-lib` to build your own CLI app, be sure to include the following in your `Cargo.toml` to enable the `doctor` and `powerpoint` features[^features]
  ```toml
  [dependencies]
  acorn-lib = { version = "0.1.52", features = ["doctor", "powerpoint"] }
  ```


[^pid]: Persistent Identifiers (PIDs) supported by `acorn-lib` include [DOI](https://www.doi.org/)s, [ORCID](https://orcid.org/)s, [RAiD](https://raid.org/)s, [ROR](https://ror.org/)s, and [ARK](https://arks.org)s.
[^version]: Check [crates.io](https://crates.io/crates/acorn-lib) for the latest version
[^features]: See the [Cargo.toml](https://code.ornl.gov/research-enablement/acorn/-/blob/main/acorn-lib/Cargo.toml#L83) for more details.