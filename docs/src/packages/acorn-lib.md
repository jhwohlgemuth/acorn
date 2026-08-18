# 🦀 Rust crate
[<img alt="crates.io" src="https://img.shields.io/crates/v/acorn-lib.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/acorn-lib)
[<img alt="docs.rs" src="https://img.shields.io/docsrs/acorn-lib?style=for-the-badge&color=%2300662C&logo=rust" height="20">](https://docs.rs/acorn-lib)
[<img alt="Size" src="https://img.shields.io/crates/size/acorn-lib?style=for-the-badge" height="20">](https://docs.rs/acorn-lib)
[<img alt="Downloads" src="https://img.shields.io/crates/dr/acorn-lib?style=for-the-badge" height="20">](https://docs.rs/acorn-lib)
[<img alt="Dependency Status" src="https://deps.rs/crate/acorn-lib/latest/status.svg?style=for-the-badge" height="20">](https://deps.rs/crate/acorn-lib/latest)

The ACORN CLI application is built on top of the `acorn-lib` Rust crate. The `acorn-lib` library provides core functionalities for working with ACORN [schemas](../concepts/acorn.md), processing [persistent identifiers through their lifecycle](../concepts/persistent-identifiers.md), and generating artifacts.

## AI And Agent APIs

The library also provides building blocks for optional AI-enabled applications:

- `acorn::schema::agent` models language models, providers, benchmark harnesses, prompt files, and agent-oriented OpenCode configuration.
- Embedded prompt templates support summarization, translation, teaching, claim extraction, and gap finding.
- `acorn::io::api::openai` provides an OpenAI-compatible client surface for completions, chat completions, responses, embeddings, model discovery, and related endpoints.

These APIs complement ACORN's deterministic schema and artifact workflows; applications decide when and how a model is invoked.

> [!IMPORTANT]
> When ACORN validates a PID, it does not rely only on recognition patterns. It also validates parsed structure and, for ISBNs[^isbn], ISNIs[^isni], ORCIDs[^orcid], and RORs[^ror], the check digits implemented for that identifier. Validation is local and does not confirm registration with an external authority.

## Installation
- Add `acorn-lib` as a dependency[^version]
  ```sh
  cargo add acorn-lib
  ```  
- Use `acorn-lib` functions in your Rust code
  ```rust
  use acorn::schema::validate::{is_ark, is_arxiv, is_doi, is_isni, is_orcid, is_ror};

  assert!(is_ark("ark:/1234/w5678").is_ok());
  assert!(is_arxiv("arXiv:2106.09685").is_ok());
  assert!(is_doi("10.11578/dc.20250604.1").is_ok());
  assert!(is_isni("https://isni.org/isni/0000000492299539").is_ok());
  assert!(is_orcid("https://orcid.org/0000-0002-2057-9115").is_ok());
  assert!(is_ror("01qz5mb56").is_ok());
  ```
- If you want to use the full capability of `acorn-lib` to build your own CLI app, be sure to enable the `doctor` and `powerpoint` features[^features]
  ```sh
  cargo add acorn-lib --features doctor,powerpoint
  ```


[^version]: Check [crates.io](https://crates.io/crates/acorn-lib) for the latest version
[^features]: See the [Cargo.toml](https://code.ornl.gov/research-enablement/acorn/-/blob/main/crates/acorn-lib/Cargo.toml#L83) for more details.
[^isbn]: [`isbn_check_digit`](https://docs.rs/acorn-lib/latest/acorn/schema/pid/fn.isbn_check_digit.html)
[^isni]: [`isni_check_digit`](https://docs.rs/acorn-lib/latest/acorn/schema/pid/fn.isni_check_digit.html)
[^orcid]: [`orcid_check_digit`](https://docs.rs/acorn-lib/latest/acorn/schema/pid/fn.orcid_check_digit.html)
[^ror]: [`ror_check_digit`](https://docs.rs/acorn-lib/latest/acorn/schema/pid/fn.ror_check_digit.html)
