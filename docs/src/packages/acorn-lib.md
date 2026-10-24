# Rust core library
[<img alt="crates.io" src="https://img.shields.io/crates/v/acorn-lib.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/acorn-lib)
[<img alt="docs.rs" src="https://img.shields.io/docsrs/acorn-lib?style=for-the-badge&color=%2300662C&logo=rust" height="20">](https://docs.rs/acorn-lib)
[<img alt="Size" src="https://img.shields.io/crates/size/acorn-lib?style=for-the-badge" height="20">](https://docs.rs/acorn-lib)
[<img alt="Downloads" src="https://img.shields.io/crates/dr/acorn-lib?style=for-the-badge" height="20">](https://docs.rs/acorn-lib)
[<img alt="Dependency Status" src="https://deps.rs/crate/acorn-lib/latest/status.svg?style=for-the-badge" height="20">](https://deps.rs/crate/acorn-lib/latest)

The ACORN CLI application uses `acorn-lib` for analysis services, persistence, provider APIs, and artifact generation. Dependency-light command execution and native filesystem, HTTP, download, source, configuration, and terminal primitives are owned by [`acorn-cmd`](acorn-cmd.md) and [`acorn-host`](acorn-host.md). Portable [schemas](../concepts/acorn.md), validation, and [persistent identifiers](../concepts/persistent-identifiers.md) are owned by `acorn-schema`.

## AI and agent APIs

The library also provides building blocks for optional AI-enabled applications:

- `acorn_schema::agent` models language models, providers, benchmark harnesses, prompt files, and agent-oriented OpenCode configuration.
- Embedded prompt templates support summarization, translation, teaching, claim extraction, and gap finding.
- `PromptHydrator` appends complete prompt data as canonical ZON by default; `PromptDataFormat::Json` provides a deterministic troubleshooting view. The inference adapters forward the hydrated Markdown body without changing their JSON wire protocols.
- With the `exec` feature, `PromptScriptHydrator` evaluates a capability-free Rhai script against caller data and returns the complete prompt body. The CLI enables this library feature in its standard build; see the [custom prompt examples](../cli-reference/commands/infer.md#custom-rhai-prompts).
- `acorn::io::api::openai` provides a client for completions, chat completions, responses, embeddings, model discovery, and related operations using supported OpenAI-style HTTP endpoints and JSON formats. It can call OpenAI's service or another compatible server, including a local one such as llama-swap; "OpenAI-compatible" describes the connection format, not the model provider.

These APIs complement ACORN's deterministic schema and artifact workflows; applications decide when and how a model is invoked.

## Runnable examples

From the workspace root, run these offline examples without an agent or API credentials:

```sh
cargo run --example confirm
cargo run --example resolve_vocabulary
cargo run --example semantic_diff
cargo run --example validate_activity
```

`confirm` needs an interactive terminal and lets you select Yes or No without changing any files. The other examples validate a bundled RAD record and show a field error, resolve research keywords against an embedded controlled vocabulary, and compare two RAD versions after canonicalization. The Rust examples and sample Rhai scripts are all in `crates/acorn-cli/examples/`.

## Installation

Add the library with its default features:

```sh
cargo add acorn-lib
```

Applications that need host analysis or PowerPoint support can enable those features explicitly:[^features]

```sh
cargo add acorn-lib --features analysis,powerpoint
```

Service applications use the dedicated portable contract and server crates:

```sh
cargo add acorn-service
cargo add acorn-server
```

See [Protobuf and gRPC](grpc.md) for the typed transport contract and security policy. The generic `acorn-lib` gRPC envelope has been removed.

Applications that need to call configured MCP tools can enable the independent `mcp-client` feature:

```sh
cargo add acorn-lib --no-default-features --features mcp-client
```

See [MCP client](mcp-client.md) for configuration, Rust examples, transport security, and operation-policy behavior. The default library feature only establishes standard-library support; inbound and outbound MCP transports remain explicit capabilities.

Native applications that only need ACORN's analysis host can omit the full default feature set:

```sh
cargo add acorn-lib --no-default-features --features analysis
```

This feature includes the analyzer's filesystem, network, document, database, and rendering adapters. It excludes the agentic server and MCP stack, system diagnostics, PowerPoint support, and shell linting. The lower-level `analyzer` feature remains the portable, allocation-only analysis kernel.

System diagnostics require `diagnostics`, and the inbound MCP server requires `agentic`. PowerPoint, chart, DuckDB, shell-lint, outbound MCP, and Polars dataframe interop (`dataframe`) likewise remain explicit. CSV check output remains part of `analysis`. This keeps unrelated native dependencies out of the default, analysis-only, and portable graphs.

For the portable data model without host services, use [`acorn-schema`](acorn-schema.md). For a small native integration that does not need analysis, use [`acorn-cmd`](acorn-cmd.md) and selected [`acorn-host`](acorn-host.md) features directly; `acorn-lib` does not re-export their APIs.

## Darwin development size gate

The workspace optimizes third-party dependencies at level 2 and `acorn-lib` at level 1 in development and test profiles. This keeps workspace code debuggable while limiting exception metadata from the native dependency graph. Run `make check-macos-unwind` on Apple Silicon to build the default CLI and its unit-test harness, compare `size -m` with `otool -l`, reject the Apple compact-unwind warning, and enforce a 14,680,064-byte `__eh_frame` limit.

The original debug CLI baseline was approximately 317 MiB with a 21,230,536-byte `__eh_frame`. On Rust 1.98.1, Cargo 1.98.1, Apple `ld` 27037.1, and arm64, the accepted incremental measurement is:

| Artifact | `__text` | `__gcc_except_tab` | `__unwind_info` | `__eh_frame` | File size |
|---|---:|---:|---:|---:|---:|
| Debug CLI | 66,268,708 | 3,909,640 | 1,970,944 | 10,451,952 | 170,009,176 |
| CLI unit-test harness | 57,944,340 | 3,259,516 | 1,687,880 | 9,026,412 | 147,983,464 |

That run took 54 seconds for the CLI and 5 seconds for the test harness after the feature graph and test dependencies were already built. The gate writes its complete machine-readable summary and raw section reports under `target/macos-unwind/`.

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
