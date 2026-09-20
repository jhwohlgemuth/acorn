# Rust core library
[<img alt="crates.io" src="https://img.shields.io/crates/v/acorn-lib.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/acorn-lib)
[<img alt="docs.rs" src="https://img.shields.io/docsrs/acorn-lib?style=for-the-badge&color=%2300662C&logo=rust" height="20">](https://docs.rs/acorn-lib)
[<img alt="Size" src="https://img.shields.io/crates/size/acorn-lib?style=for-the-badge" height="20">](https://docs.rs/acorn-lib)
[<img alt="Downloads" src="https://img.shields.io/crates/dr/acorn-lib?style=for-the-badge" height="20">](https://docs.rs/acorn-lib)
[<img alt="Dependency Status" src="https://deps.rs/crate/acorn-lib/latest/status.svg?style=for-the-badge" height="20">](https://deps.rs/crate/acorn-lib/latest)

The ACORN CLI application uses `acorn-lib` for host capabilities such as filesystem and network I/O, analysis services, persistence, and artifact generation. Portable [schemas](../concepts/acorn.md), validation, and [persistent identifiers](../concepts/persistent-identifiers.md) are owned by the separate `acorn-schema` crate.

## AI and agent APIs

The library also provides building blocks for optional AI-enabled applications:

- `acorn_schema::agent` models language models, providers, benchmark harnesses, prompt files, and agent-oriented OpenCode configuration.
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

Service applications can enable the versioned Protobuf/gRPC client and server modules with the `grpc` feature:

```sh
cargo add acorn-lib --features grpc
```

See [Protobuf and gRPC](grpc.md) for the transport contract, client example, security policy, and its role in the ACP and MCP architecture.

Native applications that only need ACORN's analysis host can omit the full default feature set:

```sh
cargo add acorn-lib --no-default-features --features analysis
```

This feature includes the analyzer's filesystem, network, document, database, and rendering adapters. It excludes the agentic server and MCP stack, system diagnostics, PowerPoint support, and shell linting. The lower-level `analyzer` feature remains the portable, allocation-only analysis kernel.

For the portable data model without host services, use [`acorn-schema`](acorn-schema.md).

## Immutable updates

The `acorn::With` derive adds consuming `with_*` methods for immutable post-construction updates. Use builders for initial construction and `with_*` methods when deriving a changed value from an existing one:

```rust
use acorn::With;

#[derive(Clone, With)]
struct ExportOptions {
    format: String,
    #[with(public, some)]
    label: Option<String>,
    #[with(private)]
    strict: bool,
}

let options = ExportOptions {
    format: "json".to_string(),
    label: None,
    strict: false,
};
let strict_yaml = options.with_format("yaml").with_label("archive").with_strict(true);

assert_eq!(strict_yaml.format, "yaml");
assert_eq!(strict_yaml.label.as_deref(), Some("archive"));
assert!(strict_yaml.strict);
```

Generated methods use the struct's visibility unless a field specifies `#[with(public)]` or `#[with(private)]`. For a field written as `Option<T>`, `#[with(some)]` accepts an inner value and stores it as `Some`; owned values accept `impl Into<T>`, while references retain their exact type so dereference coercions still work. A `some` setter does not accept `None` or provide a separate clearing method. Combine `some` with a visibility option as shown above. Use `#[with(skip)]` when a field needs validation, coordinated updates, or another custom setter.

[^features]: See the [Cargo.toml](https://code.ornl.gov/research-enablement/acorn/-/blob/main/crates/acorn-lib/Cargo.toml#L83) for more details.
