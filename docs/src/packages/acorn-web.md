# WebAssembly development

The `acorn-wasm` crate exposes ACORN's portable Research Activity validation and readability analysis to JavaScript. It operates entirely on supplied in-memory values and does not use filesystems, processes, databases, or networks.

## Installation

Install the Rust toolchain and add the WASM target:

```sh
rustup target add wasm32-unknown-unknown
```

Install `wasm-pack`, clone the ACORN repository, then build and test the package from its root:

```sh
cargo install wasm-pack --locked
make build-wasm
```

The generated web package is written to `target/acorn-wasm/pkg`. A runnable WebMCP demo is staged in `target/acorn-wasm/webmcp`. Maintainers can check the complete native and WASM portability matrix, generated package, JavaScript-facing tests, and host-dependency deny policy with:

```sh
make check-portable
```

The lower-level compile checks are:

```sh
cargo check -p acorn-analysis --no-default-features --features alloc --target wasm32-unknown-unknown
cargo check -p acorn-service --no-default-features --features alloc --target wasm32-unknown-unknown
cargo check -p acorn-wasm --target wasm32-unknown-unknown
```

Rust projects that compile to WASM can select the portable Rust surfaces directly:

```sh
cargo add acorn-analysis --no-default-features --features alloc
cargo add acorn-service --no-default-features --features alloc
```

## JavaScript API

Import the generated web module and initialize it before calling the bindings:

```js
import init, { checkReadability, validateResearchActivity } from "./target/acorn-wasm/pkg/acorn.js";

await init();

const validation = validateResearchActivity(JSON.stringify(researchActivity));
const readability = checkReadability("Supplied text to assess.", "fkgl");
```

`validateResearchActivity` returns `{ valid, value, diagnostics }`. Parse and validation diagnostics retain their original order and include their kind, path, message, and validation details when available.

`checkReadability` accepts the canonical metrics `ari`, `cli`, `fkgl`, `fres`, `gfi`, `lix`, and `smog`. It returns the ordered checks and the category summary produced from those checks. An optional URI may be supplied as the third argument.

## WebMCP browser tools

The generated package includes an opt-in adapter for the experimental browser WebMCP API. Importing it has no side effects: the application must initialize WASM, create the adapter with the generated bindings, and explicitly install its tools.

```js
import init, { checkReadability, validateResearchActivity } from "./pkg/acorn.js";
import { createAcornWebMcpAdapter } from "./pkg/webmcp.js";

await init();

const adapter = createAcornWebMcpAdapter({
    checkReadability,
    validateResearchActivity,
});
const installation = adapter.install(document.modelContext);

console.log(await installation.ready);
// Call installation.dispose() when the page no longer provides the tools.
```

The adapter registers two read-only tools:

| Tool | Input | Result |
|---|---|---|
| `acorn_validate_research_activity` | `{ json: string }` | Validity plus ordered diagnostics; parsed research data is never echoed |
| `acorn_check_readability` | `{ text: string, metric: "ari" | "cli" | "fkgl" | "fres" | "gfi" | "lix" | "smog" }` | The selected readability check and category summary |

Both schemas reject unknown fields. The adapter limits JSON input to 256 KiB, prose input to 16 KiB, output to 256 KiB, and returned diagnostics to the first 100. It permits only one active invocation per adapter and returns structured, bounded errors for invalid input, cancellation, contention, or execution failures. ACORN runs both operations locally. The adapter makes no network requests and does not persist supplied text or JSON, but a browser agent may send tool inputs and outputs to its configured model provider.

Cancellation is checked before and after each operation. The current WASM bindings are synchronous, so cancellation cannot interrupt a calculation already running on the browser's main thread. On the reference browser and machine, the native smoke recorded maximum warm times of 2.90 ms across 20 validation calls at 256 KiB and 15.30 ms across 20 readability calls at 16 KiB.

WebMCP is experimental and browser support is limited. In Chrome, enable the WebMCP testing flag before opening the demo. Unsupported browsers can still use the demo's manual validation and readability controls.

| Browser | Configuration | Verified behavior |
|---|---|---|
| Chrome 154.0.8037.98 on macOS | `chrome://flags/#enable-webmcp-testing` enabled | Passed native registration, discovery, schema and annotation inspection, execution, cancellation, manual parity, Permissions Policy denial, bounds, timing, and cleanup on 2026-10-08 |
| Chrome without the testing flag and browsers without `document.modelContext` | None | Manual WASM controls remain available; no tools are registered |

Chrome 154 exposes discovered schemas as serialized JSON and requires JSON-string arguments for in-page `executeTool()` calls. The smoke supports that implementation as well as the object arguments used by Chrome 155 and the current draft. This compatibility handling is only needed by in-page agents and tests; tool registration and browser-agent invocation use the adapter's normal structured inputs. The `tools` Permissions Policy defaults to same-origin access, and the adapter does not opt into cross-origin exposure.

Build and serve the demo from the repository root:

```sh
make build-wasm
python3 -m http.server 8000 --directory target/acorn-wasm
```

Then open `http://localhost:8000/webmcp/`. Add `?smoke` to run the native browser smoke checks. No model credentials are required. Call `installation.dispose()` when the page no longer owns the registered tools.

## Portable Rust functionality

The WASM build exposes Rust APIs that operate on supplied in-memory values. It currently includes:

| Capability | Available functionality |
|---|---|
| Research metadata | `ResearchActivity` and its supporting metadata, section, linked-data, and media types |
| Metadata standards | Serde-compatible CFF, DataCite, DCAT, Dublin Core, Huwise, Invenio, and text models; in-memory schema crosswalk contracts and conversions |
| Persistent identifiers | ARK, DOI, ISBN, ISNI, ORCID, RAiD, ROR, and patent parsing, recognition, normalization, component access, and checksum validation where the identifier defines one |
| Validation | Research schema validation plus helpers for identifiers, dates and timestamps, URLs, semantic versions, geographic coordinates and polygons, language and country codes, phone numbers, and media extensions |
| Hardware metadata | Architecture, accelerator, processor, memory, sensor, topology, vendor, and computing-resource models |
| Portable domain APIs | `Location`, `Repository`, `Scheme`, `Release`, `AcornError`, and `AcornResult` |
| Portable utilities | MIME and file-extension detection, ZON encoding/decoding, Crockford Base32, constant-time byte comparison, glob matching, regex-to-glob conversion, frontmatter splitting, semantic versions, byte formatting, and string normalization |
| Reference data | Compile-time vocabularies used by portable schema and validation code |
| Analyzer kernel | Check, error, Vale output, readability, and scoring models plus calculations over supplied text through the separate `analyzer` feature |

Schema types implement their normal Serde, JSON Schema, and validation traits in this configuration. Callers provide the strings, bytes, or already-deserialized values; ACORN does not fetch or read them in the WASM build.

For example, a Rust component compiled to WASM can use ACORN's identifier validation directly:

```rust
use acorn_schema::validation::rules::{arxiv, doi, isni, orcid, ror};

assert!(doi("10.11578/dc.20250604.1").is_ok());
assert!(arxiv("arXiv:2106.09685").is_ok());
assert!(isni("https://isni.org/isni/0000000492299539").is_ok());
assert!(orcid("https://orcid.org/0000-0002-2057-9115").is_ok());
assert!(ror("https://ror.org/01qz5mb56").is_ok());
```

## Boundaries

The current `wasm` feature selects allocation-capable APIs without enabling ACORN's native filesystem, process, terminal presentation, database, or network services. The separate `analyzer` feature adds only the portable analyzer kernel. CI compiles both paths and rejects host-only dependencies from the analyzer graph.

The package does not provide:

- browser HTTP or storage adapters
- native file or URL reads and writes
- analyzer host execution, document conversion, database, process, download, or doctor services
- std-only agent definitions and embedded agent assets

Portable crates remain independent of browser-specific dependencies; the WebMCP adapter and browser demo live only in `acorn-wasm`.

## macOS sandbox warnings

Restricted macOS environments may report that `xcrun` could not create a cache file while locating `MacOSX.sdk`. This is a host toolchain warning rather than a WASM compilation failure. If Cargo prints `Finished` and the command exits successfully, the portability check passed.
