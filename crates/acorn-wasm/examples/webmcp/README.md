# ACORN WebMCP demonstration

This page exercises ACORN's generated browser WASM package through a nine-tab manual interface and the experimental native WebMCP imperative API. Each existing tool tab has its own description, inputs, action, status, and output. The tenth read-only WebMCP tool, `acorn_format_metadata`, normalizes supplied Research Activity Data (RAD) JSON in memory; the full browser app provides its manual `/format` page. ACORN also performs metadata and Research Activity validation, persistent-identifier discovery and inspection, Big-O comparison, content diffing, FAIR assessment, typed metadata crosswalks, and readability analysis locally. A browser agent may still send tool inputs and outputs to its configured model provider.

The generated WASM package also exports `validateBigOList(string[])` and `compareBigOList(BigOComparisonInput[])` for ordered batches of up to 64 items. The WebMCP tool keeps the narrower single-pair contract so agents receive bounded, actionable request errors.

## Build and serve

1. Install Chrome 149 or newer. For local development, enable `chrome://flags/#enable-webmcp-testing` and relaunch Chrome.
2. From the repository root, run `make build-wasm`.
3. Serve the staged artifacts from an HTTP origin:

   ```sh
   python3 -m http.server 8000 --directory target/acorn-wasm
   ```

4. Open <http://127.0.0.1:8000/webmcp/>. Do not open the page with `file://`.

The manual buttons remain usable when WebMCP is unavailable. The capability status identifies whether the page registered its tools.

`acorn_format_metadata` accepts CFF with `format: "cff" | "json" | "yaml"` and `standard: "citation-file-format"`, DataCite or Invenio with `format: "json" | "yaml"` and `standard: "datacite" | "invenio"`, Dublin Core with `format: "json" | "json-ld" | "rdf-xml" | "turtle" | "yaml"` and `standard: "dublin-core"`, or RAD with `format: "json" | "jsonc" | "markdown" | "yaml" | "zon"` and `standard: "research-activity-data"`. The optional `fuzzy: "off" | "relaxed" | "strict"` defaults to Strict and affects RAD vocabulary matching only. The result includes formatted content in the selected syntax and whether it changed. JSONC comments are removed by normalization. Input and formatted output each have a 256 KiB UTF-8 limit. The tool has no file path or write authority; unsupported pairs and malformed metadata return errors while retaining the supplied text.

## Native smoke

Open <http://127.0.0.1:8000/webmcp/?smoke=1>. A passing result verifies native registration, discovery, strict schemas, safety annotations, structured execution, JSON serialization, manual parity, Permissions Policy denial, input bounds, the 100 ms timing gate, and cleanup. Missing native WebMCP support is a failure, not a skipped pass.

Record the test date, Chrome version/channel, enabled flag or origin-trial configuration, build command, URL, maximum warm timings, and displayed result. Also verify that hostile HTML in input or diagnostics is rendered as text and that a normal page load retains the manual fallback.

The smoke uses `getTools()` and `executeTool()` directly and requires no model-provider credentials. It accepts Chrome 154's serialized `inputSchema` and JSON-string execution arguments as well as the current draft's structured values. Cross-origin exposure is outside the initial support matrix; WebMCP's `tools` Permissions Policy defaults to the top-level and same-origin contexts used here.

The prior eight-tool reference result passed in Chrome 154.0.0.0 on macOS on 2026-10-09, with maximum warm times of 12.60 ms across 20 identifier calls at 2 KiB, 2.90 ms across 20 validation calls at 256 KiB, and 8.20 ms across 20 readability calls at 16 KiB.

The nine-tool native smoke passed on 2026-11-09 in interactive Chrome 156.0.8078.80 on macOS with `chrome://flags/#enable-webmcp-testing` enabled. After `make build-wasm`, the staged page at `http://127.0.0.1:8000/webmcp/?smoke=1` displayed `PASS: registration, discovery, schemas, annotations, execution, serialization, manual parity, permissions policy, bounds, timing, and cleanup.` Its user agent reported Chrome/156.0.0.0. Maximum warm times were 15.50 ms across 20 identifier calls at 2 KiB, 3.50 ms across 20 validation calls at 256 KiB, and 12.10 ms across 20 readability calls at 16 KiB. The smoke disposed its ACORN tool registrations after passing.

The ten-tool smoke passed on 2026-11-26 in headless Google Chrome 158.0.8127.88 on macOS with `--enable-features=WebMCP --enable-webmcp-testing`. The locally served staged package at `/webmcp/?smoke=1` reported `PASS: registration, discovery, schemas, annotations, execution, serialization, manual parity, permissions policy, bounds, timing, and cleanup.` It discovered and invoked `acorn_format_metadata` with RAD JSON. Maximum warm times across 20 calls each were 5.60 ms for identifier at 2 KiB, 1.70 ms for validation at 256 KiB, and 10.80 ms for readability at 16 KiB. The smoke disposed all ten registrations after passing.

The expanded smoke also passed RAD JSONC and YAML calls on 2026-11-26 in the same Chrome configuration, with JSONC comments removed and YAML returned in YAML syntax. The rebuilt package passed 49 adapter tests and 12 WASM package tests. The native CLI and generated WASM outputs were byte identical for both syntaxes on a fixture without neighboring media.

The Markdown and ZON formatter pass on 2026-11-27 passed real-WASM WebMCP adapter calls for both syntaxes. ZON output matched the native CLI byte for byte; Markdown retained the same parsed RAD data but ordered front matter fields differently. The browser page smoke passed Format selection and exact downloads for both syntaxes.

The CFF pass on 2026-11-27 passed real-WASM adapter calls for `.cff`, JSON, and YAML, and a focused native test compared their portable codec output with MCP preview and shared CLI normalization. The built browser smoke passed CFF selection and exact `.cff` download. Enrichment remains a native capability.

The DataCite pass on 2026-11-27 passed real-WASM adapter calls for JSON and YAML, including idempotence. A focused native test matched both portable outputs to MCP preview and shared CLI normalization. The built browser smoke passed DataCite selection, output, exact download, and draft retention. Enrichment remains a native capability.

The Invenio pass on 2026-11-27 passed real-WASM adapter calls for JSON and YAML, including idempotence. A focused native test matched both portable outputs to MCP preview and shared CLI normalization. The built browser smoke passed Invenio selection, output, exact download, and draft retention. Enrichment remains a native capability.

The structured Dublin Core pass on 2026-11-27 passed real-WASM adapter calls for JSON, JSON-LD, and YAML, including idempotence. A focused native test matched all three portable outputs to MCP preview and shared CLI normalization. The built browser smoke passed DCMI selection, JSON output, exact download, and draft retention.

The RDF Dublin Core pass on 2026-11-27 passed real-WASM adapter calls for RDF/XML and Turtle, including idempotence. A focused native test matched both portable outputs to MCP preview and shared CLI normalization. The built browser smoke passed RDF and TTL selector choices.
