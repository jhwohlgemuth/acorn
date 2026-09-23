# ZON serialization

ACORN supports ZON as a compact serialization of the JSON data model. ZON is a file format, not a research-metadata standard: the same Research Activity Data (RAD) schema and validation rules apply whether a record is stored as JSON, YAML, Markdown, or ZON.

Use the `.zonf` extension and provisional `text/zonf` media type. ACORN deliberately does not treat `.zon` as a ZON file because that extension is used by other formats.

```zon
meta{title:Example activity,keywords:"[research,metadata]"}
contact{email:researcher@example.org,name:Researcher}
```

The portable `acorn_schema::standard::zon` codec emits a deterministic strict profile: object keys and table columns are sorted, booleans use `T` and `F`, null uses `null`, line endings are LF, and output has no trailing whitespace or final newline. Decoding also accepts the selected aliases used by the pinned TypeScript reference implementation, including `@N` root-table headers, single-quoted strings, exponent numbers, and case-insensitive boolean and null aliases. Encoding always returns to the canonical form.

ACORN's profile is pinned to upstream commit [`1b5917dbbf254ba11fb217a378d8ea05a17598d2`](https://github.com/ZON-Format/zon-TS/commit/1b5917dbbf254ba11fb217a378d8ea05a17598d2). It supports JSON primitives, objects, arrays, compound arrays, and uniform-object tables. It rejects the separate labeled v2 syntax and enforces bounded document, line, depth, array, and object sizes.

For `tests/fixtures/data/valid_project/index.json`, the compact JSON data is 3,018 bytes, canonical ZON is 3,302 bytes, and the checked-in pretty JSON is 4,385 bytes. These are byte-size comparisons only; token counts depend on the selected model and tokenizer.

## Command-line use

Valid `.zonf` RAD files participate in normal discovery, checking, formatting, exporting, crosswalking, repository workflows, and GitLab review:

```sh
acorn check activity.zonf
acorn format activity.zonf
acorn export activity.json --format zon
acorn export datacite.zonf --format zon --from datacite --to dcat
```

`--format zon` writes `.zonf`. Check reports keep their existing console, JSON, Markdown, or YAML output formats; ZON support describes the checked input and does not add a ZON diagnostic-report format.

## Model-facing prompt data

Prompt instructions remain Markdown. `PromptHydrator` appends the complete serializable data object once in a fenced canonical ZON block by default. Library callers can select deterministic pretty JSON with `PromptDataFormat::Json` when troubleshooting.

This setting changes only the text supplied to the model. ACP continues to use its official JSON-RPC wire protocol, and OpenAI-compatible APIs continue to use their JSON HTTP request envelopes; both adapters forward the already hydrated prompt body verbatim.
