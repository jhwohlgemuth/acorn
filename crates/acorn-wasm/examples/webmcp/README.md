# ACORN WebMCP demonstration

This page exercises ACORN's generated browser WASM package through an eight-tab manual interface and the experimental native WebMCP imperative API. Each tool tab has its own description, inputs, action, status, and output. ACORN performs metadata and Research Activity validation, persistent-identifier discovery and inspection, content diffing, FAIR assessment, typed metadata crosswalks, and readability analysis locally. A browser agent may still send tool inputs and outputs to its configured model provider.

## Build and serve

1. Install Chrome 149 or newer. For local development, enable `chrome://flags/#enable-webmcp-testing` and relaunch Chrome.
2. From the repository root, run `make build-wasm`.
3. Serve the staged artifacts from an HTTP origin:

   ```sh
   python3 -m http.server 8000 --directory target/acorn-wasm
   ```

4. Open <http://127.0.0.1:8000/webmcp/>. Do not open the page with `file://`.

The manual buttons remain usable when WebMCP is unavailable. The capability status identifies whether the page registered its tools.

## Native smoke

Open <http://127.0.0.1:8000/webmcp/?smoke=1>. A passing result verifies native registration, discovery, strict schemas, safety annotations, structured execution, JSON serialization, manual parity, Permissions Policy denial, input bounds, the 100 ms timing gate, and cleanup. Missing native WebMCP support is a failure, not a skipped pass.

Record the test date, Chrome version/channel, enabled flag or origin-trial configuration, build command, URL, maximum warm timings, and displayed result. Also verify that hostile HTML in input or diagnostics is rendered as text and that a normal page load retains the manual fallback.

The smoke uses `getTools()` and `executeTool()` directly and requires no model-provider credentials. It accepts Chrome 154's serialized `inputSchema` and JSON-string execution arguments as well as the current draft's structured values. Cross-origin exposure is outside the initial support matrix; WebMCP's `tools` Permissions Policy defaults to the top-level and same-origin contexts used here.

The eight-tool reference result passed in Chrome 154.0.0.0 on macOS on 2026-10-09, with maximum warm times of 12.60 ms across 20 identifier calls at 2 KiB, 2.90 ms across 20 validation calls at 256 KiB, and 8.20 ms across 20 readability calls at 16 KiB.
