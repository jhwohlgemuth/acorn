# Rust core library
[<img alt="crates.io" src="https://img.shields.io/crates/v/acorn-lib.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/acorn-lib)
[<img alt="docs.rs" src="https://img.shields.io/docsrs/acorn-lib?style=for-the-badge&color=%2300662C&logo=rust" height="20">](https://docs.rs/acorn-lib)
[<img alt="Size" src="https://img.shields.io/crates/size/acorn-lib?style=for-the-badge" height="20">](https://docs.rs/acorn-lib)
[<img alt="Downloads" src="https://img.shields.io/crates/dr/acorn-lib?style=for-the-badge" height="20">](https://docs.rs/acorn-lib)
[<img alt="Dependency Status" src="https://deps.rs/crate/acorn-lib/latest/status.svg?style=for-the-badge" height="20">](https://deps.rs/crate/acorn-lib/latest)

The ACORN CLI application uses `acorn-lib` for host capabilities such as filesystem and network I/O, analysis services, persistence, and artifact generation. Portable [schemas](../concepts/acorn.md), validation, and [persistent identifiers](../concepts/persistent-identifiers.md) are owned by the separate `acorn-schema` crate.

## AI and agent APIs

The library also provides building blocks for optional AI-enabled applications:

- `acorn_schema::schema::agent` models language models, providers, benchmark harnesses, prompt files, and agent-oriented OpenCode configuration.
- Embedded prompt templates support summarization, translation, teaching, claim extraction, and gap finding.
- `acorn::io::api::openai` provides a client for completions, chat completions, responses, embeddings, model discovery, and related operations using supported OpenAI-style HTTP endpoints and JSON formats. It can call OpenAI's service or another compatible server, including a local one such as llama-swap; "OpenAI-compatible" describes the connection format, not the model provider.

These APIs complement ACORN's deterministic schema and artifact workflows; applications decide when and how a model is invoked.

## Installation

Add the library with its default features:

```sh
cargo add acorn-lib
```

Applications that need system diagnostics or PowerPoint support can enable those features explicitly:[^features]

```sh
cargo add acorn-lib --features doctor,powerpoint
```

For the portable data model without host services, use [`acorn-schema`](acorn-schema.md).

[^features]: See the [Cargo.toml](https://code.ornl.gov/research-enablement/acorn/-/blob/main/crates/acorn-lib/Cargo.toml#L83) for more details.
