---
type: Plan
title: CZML standard data model
description: Add a portable, validated acorn_schema::standard::czml model of CZML 1.0 documents, packets, time-dynamic properties, materials, and core graphics with schema-pinned conformance fixtures, then register it as the detectable, schema-checkable Standard::Czml, listed in the browser's standard picker.
difficulty: high
impact: medium
package: [acorn-schema, acorn-analysis, acorn-lib, acorn-service, acorn-cli, acorn-wasm, acorn-web]
tags: [czml, cesium, geospatial, schema, standards, wasm, browser]
status: planned
execution: phased
phase_budget_hours: 5
---

# Outcome and boundaries

`acorn_schema::standard::czml` models CZML, the JSON format CesiumJS uses for time-dynamic scenes. Typed Rust values cover a CZML document, its packets, and every property form the format allows: a scalar shorthand, a single interval, an interval list, time-tagged samples with interpolation settings, references, and deletion markers. The model decodes and re-encodes czml-writer's published examples without losing information. It validates the structural rules that can be checked within one document and compiles for the portable `alloc` target used by WASM.

The model is a faithful, lossless representation of authored CZML. It does NOT:

- merge packets that share an `id`, apply streaming update or truncation rules, or compute a resolved entity state;
- interpolate samples, evaluate references, or simulate the clock;
- fetch URIs, glTF models, images, or 3D Tiles;
- read files, parse server-sent event framing, or perform any other host I/O;
- render anything or change the browser map, which [ABW-72](../acorn-browser-workspace/index.md#abw-72--load-local-czml-on-the-cesium-globe) owns;
- crosswalk to or from RAD or any other standard, format CZML, run link, prose, quality, FAIR, or readability checks on it, or infer CZML metadata;

The owner approved registering CZML as a standard on November 26, 2026. CZM-09, CZM-11–14, and CZM-15 add `Standard::Czml` (serialized `czml`) with content and extension detection, native and portable schema checks, JSON Schema export, and semantic diff. The selectors are `acorn check`/`export`/`diff --standard czml`, the portable service, WASM, and the browser's standard picker. The owner asked on November 26, 2026 for CZML to be a top-tier standard, so CZM-15 lists it in the picker alongside CFF, CKAN, DataCite, and the others.

The 16 core graphics types are typed. The six AGI extension graphics (`agi_conicSensor`, `agi_customPatternSensor`, `agi_rectangularSensor`, `agi_groundElevationVolume`, `agi_fan`, `agi_vector`) are kept losslessly as extension JSON until optional CZM-10.

# Sources

The user pinned [Cesium wiki CZML Guide `be907125`](https://github.com/CesiumGS/cesium/wiki/CZML-Guide/be9071250e908c372f34a40ba377313b696e7cf7) (2012-07-02). It describes the concept but not the fields, so this plan uses three layers of sources:

| Role | Source and pinned revision | Use |
|---|---|---|
| Concept | Cesium wiki `CZML-Guide.md` at `be9071250e908c372f34a40ba377313b696e7cf7`, 2012-07-02 | Goals only: JSON-based, every property time-dynamic, sampled values interpolated by the client, incremental streaming, extensible, open. It names CZML Structure and CZML Content as the definition. |
| Prose definition | czml-writer wiki at `017e7dc1616ae9c893d0b5d160b9f27146f6115c` (2026-08-10): `CZML-Structure.md`, `CZML/Packet.md`, and 138 other `CZML/*.md` type pages | Interval, composite, sampled, streaming, availability, and extension rules. Plain-language type and property descriptions. |
| Normative shape | czml-writer `Schema/` at `61cd1d5a890ad955da081250849b99c5fd238921` (2026-11-05), JSON Schema draft-07, Apache-2.0 (`LICENSE.md`) | Field names, value forms, enum constants, `czmlRequiredForDisplay`, and the `Examples/*.json` conformance fixtures. |

The Cesium wiki's own `CZML-Structure.md` and `CZML-Content.md` (Cesium wiki HEAD `e6d7dfa5`) now only say "Moved to" and link to the czml-writer wiki. Where the schema and the czml-writer wiki disagree, the schema wins. The type-by-type inventory and known drift are in [schema-inventory.md](schema-inventory.md).

# Current evidence

CZM-01–07 implemented lexical time and references, value forms, 44 properties, materials, 16 graphics, packets, document validation, and a JSON codec in `crates/acorn-schema/src/standard/czml/` and `codec.rs`. The 43 focused CZML tests, strict package Clippy, alloc-only schema build, and `make check-wasm` passed on November 26. Pinned CesiumJS 1.146 source (`CzmlDataSource.js:4637–4645`, `2630–2645`) requires a first `document` packet and version 1.x; `ReferenceProperty.js:205–250` defines escaped reference delimiters. The model enforces these rules. Interval duration support remains unresolved, so validation rejects durations with `czml_time_unsupported`.

CZM-08 copied all 36 pinned examples and generated the schema inventory. The conformance tests pass numeric JSON equality, lexical time, packet order, and extension-map order. The owner accepted semantic JSON equality for ordinary object keys on November 26, 2026; typed structs may reorder those keys on re-encoding.

CZM-09 registered `Standard::Czml`, content detection, JSON/YAML schema export, and semantic diff. CZM-11 added native `.czml` and document-first `.json` classification with schema-only checks. CZM-12 accepts `--standard czml` in check, export, and diff; CZM-13 accepts it in the portable service and WASM parser. On November 26, 2026, 69 standard tests, 66 analysis tests, 10 semantic diff tests, 149 native host tests, 30 service tests, the CLI selector/check/schema-export tests, and the WASM parser test passed. Workspace check, strict all-target Clippy, `cargo fmt --all -- --check`, `make check-wasm`, and the wiki validator passed after the crosswalk guard and field-path repairs. The actual CLI process passed explicit and detected CZML check, JSON/YAML schema export, and an empty semantic patch for equal scenes. The initial in-process CLI test overflowed its default test thread stack; the production command passed and the CLI regression now runs it as a subprocess. CZM-11R repaired misplaced `clock` and `version` diagnostics to retain their field names; the packet suite and native host check over all ten malformed fixtures pass.

CZM-15 lists CZML in the browser's Metadata and FAIR standard pickers, while the crosswalk source and destination selectors omit it. The FAIR worker reports CZML as unsupported. On November 26, 2026, all 88 native `acorn-web` tests, `make web-check`, and the existing 14 WASM tests passed; `make web-build` produced the single-payload browser app. A Chromium smoke selected CZML, validated the valid scene, displayed the malformed reference diagnostic, received the FAIR unsupported message, and confirmed both crosswalk exclusions. The malformed field path and message match `acorn check --standard czml`; the CLI terminal prints generic code `OTHER`, while the browser exposes portable code `czml_reference`. The standalone debug WASM integration test crashed in the Node test harness, so it was removed; the production browser smoke exercises the actual CZML worker path.

The November 13 portfolio-wide review remains current. The remaining phases are final documentation/integration gates and optional AGI extension types if a consumer is confirmed.

# Contract decisions

- **Module layout.** `crates/acorn-schema/src/standard/czml/`: `mod.rs` (docs, re-exports, source constants), `time.rs`, `reference.rs`, `values.rs`, `property.rs`, `enums.rs`, `material.rs`, `graphics/{mod,marker,model,surface,volume}.rs`, `packet.rs`, `custom.rs`, `validate.rs`, `tests/`. No file is shared with another standard.
- **Lossless lexical forms.** Times, intervals, and references keep their authored strings. Parsing happens only during validation. Re-encoding never rewrites a time to a different but equivalent form.
- **Time validation.** A time is valid if `jiff::Timestamp` parses it, or if it is one of CZML's sentinel bounds `0000-00-00T00:00:00Z` and `9999-12-31T24:00:00Z`. jiff rejects both, but they are the documented default `availability`. An interval is `start/end` with exactly one `/`. A date-less end such as `13:00:00Z` takes its date from the start. Durations (`P…`) and leap seconds stay unresolved until the CesiumJS check. Until then, validation rejects them with the stable code `czml_time_unsupported`, so the decision can't drift silently.
- **Sampled versus constant values.** Fixed-arity value arrays are length-dispatched as the czml-writer wiki specifies. An array of exactly *N* is constant. An array whose length is a multiple of *N*+1 holds samples of time plus *N* components. Any other length is a decode error naming the arity. `DoubleValue` and `IntegerValue` also accept a bare number and a one-element array. List values (`Cartesian3ListValue`, etc.) are flat multiples of *N* with no time tags. RGBA components must be integers from 0 to 255.
- **Property envelope.** Every property type accepts three JSON shapes. An array is always an interval list. An object is one interval. A bare scalar is accepted only for property types whose value is a string, number, or boolean (`Boolean`, `Double`, `Integer`, `String`, `Uri`, `Font`, and enum properties). A hand-written `Deserialize` dispatches on the JSON type. `#[serde(untagged)]` is not used, because it loses eserde's per-field diagnostics.
- **Shared base fields.** `InterpolatableProperty` contributes `epoch`, `interpolationAlgorithm`, `interpolationDegree`, and the forward/backward extrapolation type and duration. `DeletableProperty` contributes `delete`. Serde can't combine `flatten` with `deny_unknown_fields`, so a private `macro_rules!` in `property.rs` emits each of the 44 core property structs with explicit base fields, the type's value keys, `///` docs passed as macro input, and a validation rule allowing at most one value form. A property with only interpolation settings or only `delete: true` is valid, because a streamed update may change only those.
- **Strictness.** `deny_unknown_fields` applies to every struct below the packet. `Packet` does not deny unknown keys. Unknown top-level keys, including the AGI extensions until CZM-10, go into an ordered `extensions` map of `serde_json::Value` and are re-encoded unchanged. CZML's documented extension point is custom top-level properties. If eserde can't flatten into a map, CZM-07 writes a map-visitor `Deserialize` for `Packet` instead.
- **Required-for-display.** Fields marked `czmlRequiredForDisplay` are `Option` and are not enforced per packet. Enforcing them needs the merge semantics this plan excludes.
- **Document rules.** `Document` is `Vec<Packet>` with an order-preserving codec. Validation runs within one document: the first packet has `id: "document"` and `version` 1.x; `version` and `clock` appear only on that packet; samples within one property interval are in increasing time; a `delete: true` packet has an `id`. Duplicate `id`s are valid. The first-packet and version rules follow CesiumJS 1.146 `CzmlDataSource.js:4637–4645` and `2630–2645`.
- **Codec.** `DocumentCodec for czml::Document`, with `NAME = "CZML document"`, decodes JSON only, through `eserde::json::from_str`. It encodes compact JSON, preserving packet and extension order; ordinary typed-object keys may be reordered under the accepted semantic-equality criterion. No `MimeType::Czml` is added, because that would change `acorn-core` and CZML has no registered media type.
- **JSON Schema.** Every public type implements `JsonSchema`. Hand-written serde types get hand-written schemas that describe the same `oneOf` shapes. The generated document schema is exported as JSON or YAML by the standard schema selector.
- **Standard identity.** The variant is `Standard::Czml`, serialized and displayed as `czml`. The CLI and WASM accept `czml` only, with no aliases. Adding the variant is additive: existing serialized names don't change.
- **Detection.** When the CLI is in its default RAD mode, any path with a `.czml` extension (case-insensitive) is classified as CZML. A `.json` file or semantic-diff input is CZML only if it is a non-empty top-level array whose first element is an object with `"id": "document"`. The detector is a public `Standard::detect_value(&Value) -> Option<Standard>` in `acorn-schema`. It checks for CZML first and otherwise defers to `detect_object`, and the diff module's array branch calls it. An explicit `--standard czml` skips detection. Detection doesn't need the document packet to have a `version`, so documents that omit it still reach schema validation and get a precise diagnostic.
- **Compatibility note.** Before this change, `.czml` files and JSON arrays starting with a `document` packet were classified as RAD and failed RAD validation. After it, they get CZML schema checks. Release notes should mention this change in behavior. No previously valid RAD input can match the detector, because RAD has no `id: "document"` array shape. A test proves this against the existing RAD fixtures.
- **Supported operations.** `supports_check` returns true only for `CheckCategory::Schema`. Link checks stay unsupported, so checking a scene never fetches its images, glTF models, or tilesets, and `offline` mode needs no special case. Every crosswalk to or from CZML returns `unsupported_crosswalk`. The CLI's existing wildcard arm already rejects formatting. Schema export emits `schemars` JSON or YAML for `czml::Document`. Semantic diff parses both sides as `czml::Document` and compares the re-encoded JSON.

- **Fixtures and licensing.** czml-writer `Schema/Examples/*.json` (36 files, Apache-2.0) are copied byte-for-byte into `tests/fixtures/schema/czml/czml-writer/`. A `README.md` beside them records the source commit, the license, and the attribution `NOTICE`. ACORN-authored fixtures are fictional and contain no real facility coordinates or private data.

# Dependencies and ownership

- CZM-01–09, CZM-11–13, and CZM-15 are implemented. CZM-14 owns final documentation and workspace gates. CZM-10 remains optional until an AGI extension consumer is confirmed.
- [ABW-72](../acorn-browser-workspace/index.md#abw-72--load-local-czml-on-the-cesium-globe) owns the independent TypeScript CZML preflight. A later WASM validation export needs its own plan before a dependency edge is added.

# Acceptance criteria

- `acorn_schema::standard::czml::Document` and its value types decode all 36 pinned czml-writer examples, each as the type its shape implies. Decoding and re-encoding gives semantically equal JSON with numeric equivalence (`0` and `0.0` compare equal), packet and extension-map order, and lexical times unchanged. Ordinary object keys may be reordered.
- An inventory generated from czml-writer `Schema/` at `61cd1d5a` matches the `schemars` property names of every typed CZML struct, apart from an explicit, reviewed exclusion list (the AGI extensions until CZM-10).
- Malformed fixtures produce stable diagnostic codes with eserde field paths: wrong arity, a non-integer RGBA component, an unknown sub-property, a composite value outside an object, an invalid time or interval, a malformed reference, out-of-order samples, and a misplaced `version` or `clock`.
- `cargo nextest run --package acorn-schema czml` passes. `cargo clippy --workspace --no-deps --all-targets` adds no warnings. `cargo fmt --check` is clean for the changed files. `make check-wasm` passes. Every public item has `///` docs.
- New code has no `unsafe`, no `?` operators, no early returns, and no bare `.unwrap()` outside tests.
- `Standard::Czml` serializes as `czml`. `acorn check` with no `--standard` classifies `scene.czml` and a document-first `.json` array as CZML and reports only the schema category as executed. `acorn check --standard czml` reports the malformed fixtures' diagnostics. `acorn export --schema --standard czml` prints the JSON schema, and `--format yaml` prints the YAML one. `acorn diff` on two CZML documents produces a semantic patch. Every crosswalk involving CZML fails with the standard "unsupported crosswalk" error.
- The portable service's `validate_standard` and the WASM `parse_standard` accept `czml`. Every existing RAD fixture still classifies as RAD.
- The browser's standard picker lists CZML for the Standard and FAIR tools but not as a crosswalk source or destination. Validating a CZM-08 fixture in the browser's Standard tool returns the same validity and malformed field path/message as `acorn check --standard czml`; its portable code is `czml_reference`, while the CLI terminal renders generic `OTHER`.

# Execution phases

## CZM-14 — Document the standard and run integration gates

- Status: ready
- Budget: 5 active hours maximum
- Prerequisites: No concurrent implementation editing the workspace, as AGENTS.md's concurrent-implementation rules require.
- Exact write set: `README.md`, `CommandLineHelp.md` (generated), `crates/acorn-schema/src/standard/czml/mod.rs` (module docs only), `.wiki/features/czml-standard-model/index.md`
- Primary integration lock: Documentation generation
- Outcome: A short README section covers CZML schema checking, detection rules, supported categories, schema export, diff, the unsupported operations, and the change in classification. It links to czml-writer's schema at the pinned commit. `make help-cli` regenerates `CommandLineHelp.md`. The module docs point to the public standard surface.
- Focused validation: `make lint`, `make test`, and `make check-wasm`. Also `cargo run --bin acorn -- check` on the CZM-08 valid and malformed fixtures, with and without `--standard czml`, and `cargo run --bin acorn -- export --schema --standard czml`.
- Handoff: Retire the plan if CZM-10 is dropped. Otherwise leave only CZM-10.

## CZM-10 — Type the AGI extension graphics

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: A confirmed consumer. The extensions are in czml-writer `Schema/Extensions/AGI/` and are not rendered by open-source CesiumJS, which is unverified beyond the schema's `agi_` prefix.
- Exact write set: `crates/acorn-schema/src/standard/czml/graphics/agi.rs` (new), `crates/acorn-schema/src/standard/czml/graphics/mod.rs`, `crates/acorn-schema/src/standard/czml/packet.rs`, `crates/acorn-schema/src/standard/czml/property.rs` (`Direction`, `DirectionList`, `SensorVolumePortionToDisplay`), `crates/acorn-schema/src/standard/czml/enums.rs`, `crates/acorn-schema/src/standard/czml/tests/conformance.rs`, `tests/fixtures/schema/czml/schema-inventory.json`
- Primary integration lock: CZML graphics model (plan-local)
- Outcome: The six `agi_*` packet properties are typed and removed from the exclusion list. Their keys are no longer routed to `extensions`.
- Focused validation: `cargo nextest run --package acorn-schema czml::tests::conformance` with an empty exclusion list, and `make check-wasm`.
- Handoff: Retire the plan once CZM-14 is complete.
