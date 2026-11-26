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

Reviewed November 26, 2026 against `3eddc62b` and the working tree. Concurrent staged wiki, daemon, and CLI edits are unrelated and were preserved. This is a targeted addition. The November 13 portfolio-wide review remains current. No CZML code was written or tested in this pass.

| Verdict | Inspected source and observation | Implication |
|---|---|---|
| verified | `git grep -i czml` at `3eddc62b`: CZML appears only in `.wiki/features/acorn-browser-workspace/` (ABW-72 and its evidence). No Rust, TypeScript, or fixture code references CZML. | The module is new. No existing contract constrains it. |
| verified | `crates/acorn-schema/src/standard/mod.rs`: `pub mod` list (cff, ckan, crosswalk, datacite, dcat†, dublin_core, huwise†, invenio, modelkit, ocp, rdf, relation, text, zon; † = `std` only). `Standard` enum with `detect_object(&Map<String, Value>)`. | Registering `czml` is a one-line `mod.rs` change under the Schema roots lock. `detect_object` only accepts a JSON object, but a CZML document is a top-level array, so CZM-09 adds a separate array-aware detector. |
| verified | Exhaustive matches on `Standard` with no wildcard arm, at `3eddc62b`: `acorn-analysis/src/check.rs` `supports_check` (line 928) and `StandardCrosswalk::crosswalk` (around line 1052); `acorn-lib/src/analyzer/host/service/mod.rs` analyzer dispatch (line 360); `acorn-lib/src/io/export/schema.rs` schema export (line 29); `acorn-lib/src/analyzer/diff/mod.rs` typed parse (line 207). The `Oci` variant (`53bc85fa`, 2026-10-07) shows the precedent: explicit `unsupported` arms where support is absent. | Adding the variant does not compile until every one of these matches has a CZML arm. CZM-09 must leave `cargo check --workspace --all-targets` clean, including any match not listed here. |
| verified | Name parsers that match on strings: `acorn-cli/src/cli/parse.rs::parse_standard` (line 266; the error text lists the accepted names), `acorn-wasm/src/lib.rs::parse_standard` (line 874). The help text `Supported standards: …` appears five times in `acorn-cli/src/cli/arguments/mod.rs` (lines 642, 1057, 1083, 1101, 1159) and is generated into `CommandLineHelp.md`. | `--standard czml` and WASM selection need explicit arms, updated help text, and a `make help-cli` regeneration. The compiler does not catch these. |
| verified | `acorn-lib/src/analyzer/host/service/mod.rs::StandardHostExt::from_path` (line 251) classifies paths by file name and then by `MimeType`. JSON content detection only accepts objects (`value.as_object().map(Self::from)`). `acorn-core` `MimeType::from` has no `czml` extension, so `scene.czml` falls through to `ResearchActivityData`. `acorn-lib/src/analyzer/diff/mod.rs::detect_value` detects an array by detecting each element and requiring them all to agree. | Detection must check for a CZML document before the per-element logic, and must map the `.czml` extension in `from_path` itself so `acorn-core` doesn't change. |
| verified | Native schema checks for a standard need three things. `impl InputOutput` in `acorn-lib/src/io/adapter/mod.rs` provides `read` and `read_json`. The `impl_schema_check!` list (`acorn-lib/src/analyzer/host/runtime/mod.rs:177`) requires `T: Validate`. `impl Analysis` in the same file supplies `standard()` and `output_path`. The portable `acorn-service/src/service.rs::validate_standard` dispatches through `validate_decoded::<T>` and has a wildcard "not available" arm. | CZML schema checks reuse these existing seams. No new trait or check category is needed. |
| verified | `crates/acorn-schema/src/lib.rs`: `#![cfg_attr(not(feature = "std"), no_std)]`, `extern crate alloc`. `Cargo.toml` features `alloc`, `std`, `wasm = ["alloc"]`. Dependencies include `serde`, `serde_json`, `schemars` 1.0 (no default features), `serde_with`, `jiff` (`alloc`), `eserde`, `bon`, `strum`, `thiserror`. | Every CZML type must compile without `std`. No new dependency is needed. |
| verified | `crates/acorn-schema/src/codec.rs`: the `DocumentCodec` trait (`NAME`, `decode`, `encode`). The Cff and RAiD JSON decoders call `eserde::json::from_str` and map errors to `CodecDiagnostic` with paths. | `czml::Document` implements `DocumentCodec` for `MimeType::Json` in the same style. |
| verified | `crates/acorn-lib/src/io/config/application.rs:356–388`: `eserde::Deserialize` structs mark fields with hand-written serde types as `#[eserde(compat)]`. | CZML's hand-written array and envelope deserializers can sit inside eserde-derived parents. CZM-02 confirms this with a test before CZM-03 depends on it. |
| verified | `crates/acorn-schema/src/standard/tests/ocp.rs` loads `tests/fixtures/schema/ocp/contribution.json` with `include_str!`. `tests/mod.rs` has a `canonical_json` helper. | CZML fixtures go under `tests/fixtures/schema/czml/`, and tests live in a module-local `czml/tests/` directory, as `modelkit` and `zon` do. |
| verified | czml-writer wiki `CZML-Structure.md`: a document is one JSON array of packets. A property value is an interval list (array), a single interval object, or a bare string, number, or boolean when it is constant for all time. Composite values must be wrapped in an object. Samples use `[time, v…]` with ISO 8601 strings or seconds from `epoch`. `availability` is one interval string or an array of them. Custom properties should carry a prefix. | These rules define the property envelope (CZM-03) and the sample encoding (CZM-02). |
| verified | czml-writer `Schema/Billboard.json`: `image` carries `czmlRequiredForDisplay: true`, not a JSON Schema `required` array. | Fields the wiki marks "Required" are only required once packets are merged for display. Streaming lets a later packet add a field, so a single packet can't be rejected for omitting one. |
| verified | czml-writer `Schema/Packet.json` at `61cd1d5a`: 33 properties, including `agi_groundElevationVolume`. czml-writer wiki `CZML/Packet.md` at `017e7dc1` lists 32 and omits it. | The wiki trails the schema. CZM-08 checks the Rust field set against an inventory generated from the pinned schema, not from the wiki. |
| verified | The 13 enum value schemas under czml-writer `Schema/Values/` at `61cd1d5a` match the czml-writer wiki constants exactly (listed in [schema-inventory.md](schema-inventory.md#enumerations)). | The schema's enum constants are authoritative. CesiumJS runtime enums may have more values than the schema. That is unverified and out of scope. |
| unresolved | Whether CesiumJS requires the first packet to be `id: "document"` with `version`, how it escapes `#` and `.` in reference strings, and whether it accepts ISO 8601 durations or partial end times in intervals. The czml-writer wiki shows a partial end (`2012-04-30T12:00:00Z/13:00:00Z`) without defining the rule. | CZM-01 and CZM-07 make these checks follow the decisions below and test both sides of each boundary. Before freezing them, inspect CesiumJS `CzmlDataSource.js`, `ReferenceProperty.js`, and `TimeInterval.fromIso8601` at a pinned release tag and record the locator. |

# Contract decisions

- **Module layout.** `crates/acorn-schema/src/standard/czml/`: `mod.rs` (docs, re-exports, source constants), `time.rs`, `reference.rs`, `values.rs`, `property.rs`, `enums.rs`, `material.rs`, `graphics/{mod,marker,model,surface,volume}.rs`, `packet.rs`, `custom.rs`, `validate.rs`, `tests/`. No file is shared with another standard.
- **Lossless lexical forms.** Times, intervals, and references keep their authored strings. Parsing happens only during validation. Re-encoding never rewrites a time to a different but equivalent form.
- **Time validation.** A time is valid if `jiff::Timestamp` parses it, or if it is one of CZML's sentinel bounds `0000-00-00T00:00:00Z` and `9999-12-31T24:00:00Z`. jiff rejects both, but they are the documented default `availability`. An interval is `start/end` with exactly one `/`. A date-less end such as `13:00:00Z` takes its date from the start. Durations (`P…`) and leap seconds stay unresolved until the CesiumJS check. Until then, validation rejects them with the stable code `czml_time_unsupported`, so the decision can't drift silently.
- **Sampled versus constant values.** Fixed-arity value arrays are length-dispatched as the czml-writer wiki specifies. An array of exactly *N* is constant. An array whose length is a multiple of *N*+1 holds samples of time plus *N* components. Any other length is a decode error naming the arity. `DoubleValue` and `IntegerValue` also accept a bare number and a one-element array. List values (`Cartesian3ListValue`, etc.) are flat multiples of *N* with no time tags. RGBA components must be integers from 0 to 255.
- **Property envelope.** Every property type accepts three JSON shapes. An array is always an interval list. An object is one interval. A bare scalar is accepted only for property types whose value is a string, number, or boolean (`Boolean`, `Double`, `Integer`, `String`, `Uri`, `Font`, and enum properties). A hand-written `Deserialize` dispatches on the JSON type. `#[serde(untagged)]` is not used, because it loses eserde's per-field diagnostics.
- **Shared base fields.** `InterpolatableProperty` contributes `epoch`, `interpolationAlgorithm`, `interpolationDegree`, and the forward/backward extrapolation type and duration. `DeletableProperty` contributes `delete`. Serde can't combine `flatten` with `deny_unknown_fields`, so a private `macro_rules!` in `property.rs` emits each of the 44 core property structs with explicit base fields, the type's value keys, `///` docs passed as macro input, and a validation rule allowing at most one value form. A property with only interpolation settings or only `delete: true` is valid, because a streamed update may change only those.
- **Strictness.** `deny_unknown_fields` applies to every struct below the packet. `Packet` does not deny unknown keys. Unknown top-level keys, including the AGI extensions until CZM-10, go into an ordered `extensions` map of `serde_json::Value` and are re-encoded unchanged. CZML's documented extension point is custom top-level properties. If eserde can't flatten into a map, CZM-07 writes a map-visitor `Deserialize` for `Packet` instead.
- **Required-for-display.** Fields marked `czmlRequiredForDisplay` are `Option` and are not enforced per packet. Enforcing them needs the merge semantics this plan excludes.
- **Document rules.** `Document` is `Vec<Packet>` with an order-preserving codec. Validation runs within one document: `version` and `clock` may appear only on the packet whose `id` is `document`; `version` matches `<major>.<minor>` with major `1`; samples within one property interval are in increasing time; a `delete: true` packet has an `id`. Duplicate `id`s are valid. Whether the document packet must come first is decided when CZM-07 records the CesiumJS evidence. If that is still unresolved, the check is a separate, opt-in `Document::validate_document_packet_first()` and not part of `Validate`.
- **Codec.** `DocumentCodec for czml::Document`, with `NAME = "CZML document"`, decodes JSON only, through `eserde::json::from_str`. It encodes compact JSON in source order. No `MimeType::Czml` is added, because that would change `acorn-core` and CZML has no registered media type.
- **JSON Schema.** Every public type implements `JsonSchema`. Hand-written serde types get hand-written schemas that describe the same `oneOf` shapes. Exporting the generated schema stays internal to tests until CZM-09.
- **Standard identity.** The variant is `Standard::Czml`, serialized and displayed as `czml`. The CLI and WASM accept `czml` only, with no aliases. Adding the variant is additive: existing serialized names don't change.
- **Detection.** When the CLI is in its default RAD mode, any path with a `.czml` extension (case-insensitive) is classified as CZML. A `.json` file or semantic-diff input is CZML only if it is a non-empty top-level array whose first element is an object with `"id": "document"`. The detector is a public `Standard::detect_value(&Value) -> Option<Standard>` in `acorn-schema`. It checks for CZML first and otherwise defers to `detect_object`, and the diff module's array branch calls it. An explicit `--standard czml` skips detection. Detection doesn't need the document packet to have a `version`, so documents that omit it still reach schema validation and get a precise diagnostic.
- **Compatibility note.** Before this change, `.czml` files and JSON arrays starting with a `document` packet were classified as RAD and failed RAD validation. After it, they get CZML schema checks. Release notes should mention this change in behavior. No previously valid RAD input can match the detector, because RAD has no `id: "document"` array shape. A test proves this against the existing RAD fixtures.
- **Supported operations.** `supports_check` returns true only for `CheckCategory::Schema`. Link checks stay unsupported, so checking a scene never fetches its images, glTF models, or tilesets, and `offline` mode needs no special case. Every crosswalk to or from CZML returns `unsupported_crosswalk`. The CLI's existing wildcard arm already rejects formatting. Schema export emits `schemars` JSON or YAML for `czml::Document`. Semantic diff parses both sides as `czml::Document` and compares the re-encoded JSON.

- **Fixtures and licensing.** czml-writer `Schema/Examples/*.json` (36 files, Apache-2.0) are copied byte-for-byte into `tests/fixtures/schema/czml/czml-writer/`. A `README.md` beside them records the source commit, the license, and the attribution `NOTICE`. ACORN-authored fixtures are fictional and contain no real facility coordinates or private data.

# Dependencies and ownership

- No other plan blocks this one. CZM-01 takes the shared Schema roots lock for its one-line `standard/mod.rs` registration. CZM-07 takes it again for `codec.rs`. The other phases write only inside `standard/czml/`, `tests/fixtures/schema/czml/`, and this plan.
- [ABW-72](../acorn-browser-workspace/index.md#abw-72--load-local-czml-on-the-cesium-globe) is independent. Its TypeScript preflight (2 MiB, 1,000 packets, document packet, external-resource rejection) doesn't wait for this model. A later, separately planned WASM validation export could replace that preflight. No edge is added until such a plan exists.
- The owner approved `Standard::Czml` on November 26, 2026, which satisfies AGENTS.md's ask-first rule for the vocabulary change. CZM-09 needs CZM-08's accepted model and fixtures. It claims Schema roots and must find the Analyzer/check contract lock free, because it edits `check.rs` and the analyzer dispatch. CZM-11 claims Analyzer/check contract, CZM-12 claims CLI contract, and CZM-13 claims Service/RPC contract. CZM-12 and CZM-13 have disjoint write sets and can run in parallel after CZM-11. CZM-15 follows CZM-13 and claims the browser application shell lock on `acorn-web/src/ui.rs`, which serializes with ABW, browser format parity, browser performance, shadcn component, and USWDS phases that edit that file. CZM-14 runs last and owns workspace-wide validation and the generated help.

# Acceptance criteria

- `acorn_schema::standard::czml::Document` and its value types decode all 36 pinned czml-writer examples, each as the type its shape implies. Decoding and re-encoding gives JSON that matches the original numerically (`0` and `0.0` compare equal) with key order and lexical times unchanged.
- An inventory generated from czml-writer `Schema/` at `61cd1d5a` matches the `schemars` property names of every typed CZML struct, apart from an explicit, reviewed exclusion list (the AGI extensions until CZM-10).
- Malformed fixtures produce stable diagnostic codes with eserde field paths: wrong arity, a non-integer RGBA component, an unknown sub-property, a composite value outside an object, an invalid time or interval, a malformed reference, out-of-order samples, and a misplaced `version` or `clock`.
- `cargo nextest run --package acorn-schema czml` passes. `cargo clippy --workspace --no-deps --all-targets` adds no warnings. `cargo fmt --check` is clean for the changed files. `make check-wasm` passes. Every public item has `///` docs.
- New code has no `unsafe`, no `?` operators, no early returns, and no bare `.unwrap()` outside tests.
- `Standard::Czml` serializes as `czml`. `acorn check` with no `--standard` classifies `scene.czml` and a document-first `.json` array as CZML and reports only the schema category as executed. `acorn check --standard czml` reports the malformed fixtures' diagnostics. `acorn export --schema --standard czml` prints the JSON schema, and `--format yaml` prints the YAML one. `acorn diff` on two CZML documents produces a semantic patch. Every crosswalk involving CZML fails with the standard "unsupported crosswalk" error.
- The portable service's `validate_standard` and the WASM `parse_standard` accept `czml`. Every existing RAD fixture still classifies as RAD.
- The browser's standard picker lists CZML for the Standard and FAIR tools but not as a crosswalk source or destination. Validating a CZM-08 fixture in the browser's Standard tool returns the same schema diagnostics as `acorn check --standard czml`.

# Execution phases

## CZM-01 — Scaffold the module with time, interval, and reference primitives

- Status: ready
- Budget: 5 active hours maximum
- Prerequisites: none. Confirm no other phase holds the Schema roots lock, and record a baseline `cargo check --package acorn-schema --no-default-features --features alloc`.
- Exact write set: `crates/acorn-schema/src/standard/mod.rs` (add `pub mod czml;` only), `crates/acorn-schema/src/standard/czml/mod.rs` (new), `crates/acorn-schema/src/standard/czml/time.rs` (new), `crates/acorn-schema/src/standard/czml/reference.rs` (new), `crates/acorn-schema/src/standard/czml/tests/mod.rs` (new), `crates/acorn-schema/src/standard/czml/tests/time.rs` (new)
- Primary integration lock: Schema roots
- Outcome: Module docs say what the model does and does not do. `pub const CZML_VERSION: &str = "1.0"` and the pinned source constants exist. The lexical-preserving types `Time`, `TimeInterval`, `TimeIntervalCollection` (string or array), `SampleTime` (ISO string or seconds offset), `Reference` (`id#property.path`, both sides non-empty), and `VelocityReference` are in place, with serde, `JsonSchema`, and `Validate` following the time-validation decision.
- Focused validation: `cargo nextest run --package acorn-schema czml::tests::time` covers full timestamps, both sentinels, the date-less end, rejected durations and leap seconds, interval separator errors, string and array availability, and reference forms. Also run the `alloc` check and package Clippy.
- Handoff: Record the CesiumJS locators for the interval and reference rules if they were inspected. Make CZM-02 ready.

## CZM-02 — Encode constant, sampled, and list value arrays

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: CZM-01
- Exact write set: `crates/acorn-schema/src/standard/czml/values.rs` (new), `crates/acorn-schema/src/standard/czml/tests/values.rs` (new), `crates/acorn-schema/src/standard/czml/mod.rs`, `crates/acorn-schema/src/standard/czml/tests/mod.rs`
- Primary integration lock: CZML value encoding (plan-local)
- Outcome: A const-generic fixed-arity value that is either constant or a list of time-tagged samples. Typed aliases for every value kind in [schema-inventory.md](schema-inventory.md#value-encodings). `DoubleValue` and `IntegerValue` with scalar shorthand. Integer RGBA and float RGBAF. Flat lists, lists of lists, reference lists, and `DoubleList`. Sequence visitors decode directly into these types, with no intermediate `serde_json::Value`. Each has a matching `JsonSchema` and arity-specific error messages. A test proves an eserde-derived parent with an `#[eserde(compat)]` field keeps per-field paths.
- Focused validation: `cargo nextest run --package acorn-schema czml::tests::values` covers each arity's constant, ISO-sampled, epoch-sampled, and invalid lengths, RGBA bounds, empty and ragged lists, and a numeric round trip. Also run the `alloc` check.
- Handoff: Record the eserde compatibility result. If it fails, add a repair phase before CZM-03 instead of switching CZM-03 to `untagged`. Make CZM-03 ready.

## CZM-03 — Add the property envelope, base fields, enumerations, and scalar properties

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: CZM-02
- Exact write set: `crates/acorn-schema/src/standard/czml/property.rs` (new), `crates/acorn-schema/src/standard/czml/enums.rs` (new), `crates/acorn-schema/src/standard/czml/tests/property.rs` (new), `crates/acorn-schema/src/standard/czml/mod.rs`, `crates/acorn-schema/src/standard/czml/tests/mod.rs`
- Primary integration lock: CZML value encoding (plan-local)
- Outcome: A generic `Property<T>` envelope: interval list, single interval, or scalar shorthand where allowed. `Interpolation` and deletion base fields. The private property macro. All 16 enumerations (13 from the schema plus `InterpolationAlgorithm`, `ExtrapolationType`, and `ReferenceFrame`), serialized as `SCREAMING_SNAKE_CASE`. The 19 deletable-only property structs: `Boolean`, `String`, `Uri`, `Font`, `DoubleList`, `Shape`, `PositionList`, `PositionListOfLists`, and the 11 enum properties. `Double` and `Integer` with interpolation settings. The rule allowing at most one value form.
- Focused validation: `cargo nextest run --package acorn-schema czml::tests::property` covers the three envelope shapes for scalar and enum properties, a composite value rejected without its object wrapper, a delete-only interval, an interpolation-only update, two value forms rejected, an unknown sub-property rejected, and every enum constant round-tripped.
- Handoff: Make CZM-04 ready.

## CZM-04 — Add the geometric, color, and orientation properties

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: CZM-03
- Exact write set: `crates/acorn-schema/src/standard/czml/property.rs`, `crates/acorn-schema/src/standard/czml/tests/property.rs`
- Primary integration lock: CZML value encoding (plan-local)
- Outcome: The 23 remaining interpolatable core properties: `Position` (with `referenceFrame` and its five value forms), `Orientation`, `ViewFrom`, `Color`, `AlignedAxis`, `EyeOffset`, `PixelOffset`, `BackgroundPadding`, `Repeat`, `LineCount`, `LineOffset`, `LineThickness`, `BoxDimensions`, `EllipsoidRadii`, `RectangleCoordinates`, `NearFarScalar`, `BoundingRectangle`, `DistanceDisplayCondition`, `Rotation`, `Scale`, `Translation`, `Articulation`, and `CustomProperty` (the 27-key union). Each is built with the CZM-03 macro and documented from its schema description.
- Focused validation: The same test target covers the wiki's `Position` examples (constant cartographic and sampled ISS cartesian), `Color` RGBA and RGBAF, `Orientation` with `velocityReference`, a `CustomProperty` with each value key, and the schemars property names for every new struct.
- Handoff: Make CZM-05 ready.

## CZM-05 — Add materials and the marker, model, path, and tileset graphics

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: CZM-04
- Exact write set: `crates/acorn-schema/src/standard/czml/material.rs` (new), `crates/acorn-schema/src/standard/czml/graphics/mod.rs` (new), `crates/acorn-schema/src/standard/czml/graphics/marker.rs` (new), `crates/acorn-schema/src/standard/czml/graphics/model.rs` (new), `crates/acorn-schema/src/standard/czml/tests/graphics.rs` (new), `crates/acorn-schema/src/standard/czml/mod.rs`, `crates/acorn-schema/src/standard/czml/tests/mod.rs`
- Primary integration lock: CZML graphics model (plan-local)
- Outcome: `Material`, with five exclusive kinds and a validation rule allowing at most one. `PolylineMaterial`, with nine kinds. The nine material structs. `Billboard`, `Label`, `Point`, `Model` (with the ordered maps `NodeTransformations` and `Articulations`), `Path`, and `Tileset`, as `bon` builders with `deny_unknown_fields` and every `czmlRequiredForDisplay` field `Option`.
- Focused validation: `cargo nextest run --package acorn-schema czml::tests::graphics` covers the wiki's Packet billboard and label example, the model node-transformation example, a material with two kinds rejected, an articulation key of the form `name stage`, and an unknown graphics field rejected.
- Handoff: Make CZM-06 ready.

## CZM-06 — Add the surface and volume graphics

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: CZM-05
- Exact write set: `crates/acorn-schema/src/standard/czml/graphics/surface.rs` (new), `crates/acorn-schema/src/standard/czml/graphics/volume.rs` (new), `crates/acorn-schema/src/standard/czml/graphics/mod.rs`, `crates/acorn-schema/src/standard/czml/tests/graphics.rs`
- Primary integration lock: CZML graphics model (plan-local)
- Outcome: The surface graphics `Corridor`, `Ellipse`, `Polygon` (with `holes`), `Polyline`, `Rectangle`, and `Wall`. The volume graphics `Box`, `Cylinder`, `Ellipsoid`, and `PolylineVolume`. Field sets match [schema-inventory.md](schema-inventory.md#graphics), including `zIndex`, `classificationType`, and the extruded-height references.
- Focused validation: The same test target covers one complete instance of each of the 10 types, polygon holes as a list of lists, wall minimum and maximum heights, and the schemars property names for each.
- Handoff: Make CZM-07 ready.

## CZM-07 — Assemble packets and documents and register the codec

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: CZM-06. Record the CesiumJS document-packet evidence or explicitly mark it unresolved.
- Exact write set: `crates/acorn-schema/src/standard/czml/packet.rs` (new), `crates/acorn-schema/src/standard/czml/custom.rs` (new), `crates/acorn-schema/src/standard/czml/validate.rs` (new), `crates/acorn-schema/src/standard/czml/tests/packet.rs` (new), `crates/acorn-schema/src/standard/czml/mod.rs`, `crates/acorn-schema/src/standard/czml/tests/mod.rs`, `crates/acorn-schema/src/codec.rs`
- Primary integration lock: Schema roots
- Outcome: `Packet` with the 27 typed schema properties and the order-preserving `extensions` map. `CustomProperties`. `Clock`. `Document`. The document-level validation rules. `DocumentCodec for Document`, using `eserde::json::from_str` with diagnostics mapped to field paths and a rejection of trailing JSON, matching the RAD decoder.
- Focused validation: `cargo nextest run --package acorn-schema czml::tests::packet` covers the wiki's document, delete, and facility packets, duplicate ids accepted, a misplaced `version` or `clock`, out-of-order samples, a `delete: true` packet without an `id`, a prefixed extension key round-tripped unchanged, an AGI key preserved as an extension, and codec diagnostics for malformed JSON.
- Handoff: Update the module docs with the final document-packet rule and its evidence. Make CZM-08 ready.

## CZM-08 — Prove conformance with pinned czml-writer fixtures

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: CZM-07
- Exact write set: `tests/fixtures/schema/czml/README.md` (new), `tests/fixtures/schema/czml/NOTICE` (new), `tests/fixtures/schema/czml/czml-writer/` (new directory holding only the 36 JSON files of czml-writer `Schema/Examples/`, copied byte-for-byte from `61cd1d5a`), `tests/fixtures/schema/czml/schema-inventory.json` (new), `tests/fixtures/schema/czml/acorn/scene.czml` (new), `tests/fixtures/schema/czml/acorn/malformed.json` (new; a map from case name to an invalid document, one entry per acceptance-criteria case), `crates/acorn-schema/src/standard/czml/tests/conformance.rs` (new), `crates/acorn-schema/src/standard/czml/tests/mod.rs`
- Primary integration lock: CZML fixtures (plan-local)
- Outcome: Each czml-writer example decodes as the type its shape implies, and round-trips. Packet fragments such as `Billboard.json` (`{"billboard": …}`) decode as `Packet`. Bare value arrays such as `Cartesian3ValueSamples.json` decode as the value type whose schema lists them in `czmlExamples`. Each example round-trips as the acceptance criteria require. `schema-inventory.json` maps each type to its property names and `czmlRequiredForDisplay` fields at `61cd1d5a`. The README records the one-off command that generated it. A test compares it with the schemars output and keeps a reviewed exclusion list. A fictional ACORN scene of a research-site marker with a time-varying instrument track, plus the malformed cases from the acceptance criteria, passes `Validate`.
- Focused validation: `cargo nextest run --package acorn-schema czml::tests::conformance`. Then run the full `czml` filter, package Clippy with `--all-targets`, `cargo fmt --check` on the changed files, and `make check-wasm`.
- Handoff: Record the exclusion list and any schema drift in [schema-inventory.md](schema-inventory.md). Make CZM-09 ready.

## CZM-09 — Add the `Standard::Czml` variant, detection, export, and diff

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: CZM-08. Schema roots free, and the Analyzer/check contract lock free for the duration of this phase.
- Exact write set: `crates/acorn-schema/src/standard/mod.rs`, `crates/acorn-schema/src/standard/tests/mod.rs`, `crates/acorn-analysis/src/check.rs`, `crates/acorn-analysis/src/tests/mod.rs`, `crates/acorn-lib/src/analyzer/host/service/mod.rs` (dispatch arm only), `crates/acorn-lib/src/io/export/schema.rs`, `crates/acorn-lib/src/analyzer/diff/mod.rs`, `crates/acorn-lib/src/analyzer/diff/tests.rs`. If `cargo check --workspace --all-targets` reports another exhaustive match, stop and add that file to this list on the plan before editing it.
- Primary integration lock: Schema roots
- Outcome: The variant `Standard::Czml`, serialized and displayed as `czml`. `Standard::detect_value` follows the detection decision. `supports_check` covers only the schema category. Every source or target crosswalk arm returns `unsupported_crosswalk`. The analyzer dispatch arm temporarily returns `standard.unsupported(&categories)` until CZM-11. Schema export serializes `czml::Document`. Diff parses CZML with `parse_typed::<czml::Document>()`, and its array branch detects CZML before comparing elements.
- Focused validation: `cargo nextest run --package acorn-schema standard::tests` covers serialization, detecting a document-first array, an empty array, an array whose first element lacks the `document` id, and every existing RAD and standard fixture still detecting as before. Also run `cargo nextest run --package acorn-analysis` for the support matrix and crosswalk rejection, `cargo nextest run --package acorn-lib analyzer::diff` for two-document CZML diff and detection, `cargo check --workspace --all-targets`, and `make check-wasm`.
- Handoff: Record any extra exhaustive match found. Make CZM-11 ready.

## CZM-11 — Run native CZML schema checks and classify `.czml` paths

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: CZM-09
- Exact write set: `crates/acorn-lib/src/io/adapter/mod.rs`, `crates/acorn-lib/src/analyzer/host/runtime/mod.rs`, `crates/acorn-lib/src/analyzer/host/service/mod.rs`, `crates/acorn-lib/src/analyzer/host/service/tests/mod.rs`, `crates/acorn-lib/src/analyzer/host/runtime/tests/mod.rs`
- Primary integration lock: Analyzer/check contract
- Outcome: `impl InputOutput for czml::Document`. `read` accepts `.czml` and `.json` as JSON, and `read_json` decodes through `DocumentCodec` so eserde diagnostics keep their paths. `czml::Document` joins `impl_schema_check!`. `impl Analysis for czml::Document` returns `Standard::Czml` and the standard project output path. `StandardHostExt::from_path` maps `.czml` to CZML before the MIME dispatch, and its JSON branch calls `Standard::detect_value`. The dispatch arm runs `analyze::<czml::Document>`.
- Focused validation: `cargo nextest run --package acorn-lib analyzer::host` covers `classify_paths` for `scene.czml`, a document-first `.json`, and a RAD `.json`. CZM-08's valid scene yields executed schema checks with no failures. Each malformed fixture yields a schema failure with a field path. Link, prose, quality, FAIR, and readability report unsupported, and no network client is constructed.
- Handoff: Make CZM-12 and CZM-13 ready.

## CZM-12 — Accept `--standard czml` in the CLI

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: CZM-11. CLI contract lock free.
- Exact write set: `crates/acorn-cli/src/cli/parse.rs`, `crates/acorn-cli/src/cli/arguments/mod.rs` (the five `Supported standards` doc lines only), `crates/acorn-cli/src/cli/tests/mod.rs`, `crates/acorn-cli/src/commands/check/tests/mod.rs`, `crates/acorn-cli/src/commands/export/tests/mod.rs`
- Primary integration lock: CLI contract
- Outcome: `parse_standard` accepts `czml`, and the unknown-standard message and the five help lines list it. `acorn check`, `acorn export --schema`, and `acorn diff` reach the CZM-09 and CZM-11 behavior through the existing commands, with no new flags. Crosswalk export with `czml` fails with the existing unsupported error.
- Focused validation: `cargo nextest run --package acorn-cli cli::tests commands::check::tests commands::export::tests` covers name parsing, a rejected-name message that includes `czml`, a check of the valid and malformed fixtures with explicit and detected standards, a JSON and YAML schema export snapshot, and crosswalk rejection.
- Handoff: Leave `CommandLineHelp.md` regeneration to CZM-14. Make CZM-14 eligible once CZM-13 is also done.

## CZM-13 — Accept CZML in the portable service and WASM

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: CZM-11. Service/RPC contract lock free.
- Exact write set: `crates/acorn-service/src/service.rs`, `crates/acorn-service/src/tests/mod.rs`, `crates/acorn-wasm/src/lib.rs`
- Primary integration lock: Service/RPC contract
- Outcome: `validate_standard` dispatches `Standard::Czml` to `validate_decoded::<czml::Document>`, with no `std` gate. The WASM `parse_standard` accepts `czml`. The WASM export list and the browser UI don't change.
- Focused validation: `cargo nextest run --package acorn-service` covers valid and malformed CZML request content and the response diagnostics. Also run `make check-wasm` and a WASM unit test for `parse_standard("czml")`.
- Handoff: Make CZM-14 eligible once CZM-12 is also done.

## CZM-15 — List CZML in the browser's standard picker

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: CZM-13. No in-progress ABW, browser format parity, browser performance, shadcn component, or USWDS phase may have `crates/acorn-web/src/ui.rs` in its write set; confirm with the browser lane owner before starting.
- Exact write set: `crates/acorn-web/src/ui.rs` (the `STANDARD_CHOICES` array only), `crates/acorn-web/tests/czml_standard.rs` (new)
- Primary integration lock: browser application shell (`acorn-web/src/ui.rs`)
- Outcome:
  - Add `(Standard::Czml, "CZML", "Cesium Language")` to `STANDARD_CHOICES`, in the existing order, and raise the array length from 11 to 12.
  - The `StandardField` crosswalk filter stays as it is, so CZML never appears as a crosswalk source or destination.
  - The FAIR tool lists CZML in the same way it lists CKAN today, and reports FAIR as unsupported for it.
  - No other browser UI, route, format list, or WASM export changes.
- Focused validation:
  - `cargo nextest run -p acorn-web --test czml_standard`. The test runs the Standard tool's worker dispatch on one valid and one malformed CZM-08 fixture, and checks the diagnostic codes against `acorn check --standard czml`.
  - `cargo nextest run -p acorn-web`, then `make web-check`.
  - A browser smoke check that selects CZML in the Standard tool and validates a fixture.
- Handoff: Remove CZM-15 and make CZM-14 eligible once CZM-12 and CZM-13 are also done.

## CZM-14 — Document the standard and run integration gates

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: CZM-12, CZM-13, and CZM-15. No concurrent implementation editing the workspace, as AGENTS.md's concurrent-implementation rules require.
- Exact write set: `README.md`, `CommandLineHelp.md` (generated), `crates/acorn-schema/src/standard/czml/mod.rs` (module docs only), `.wiki/features/czml-standard-model/index.md`
- Primary integration lock: Documentation generation
- Outcome: A short README section covers CZML schema checking, detection rules, supported categories, schema export, diff, the unsupported operations, and the change in classification. It links to czml-writer's schema at the pinned commit. `make help-cli` regenerates `CommandLineHelp.md`. The module docs point to the public standard surface.
- Focused validation: `make lint`, `make test`, and `make check-wasm`. Also `cargo run --bin acorn -- check` on the CZM-08 valid and malformed fixtures, with and without `--standard czml`, and `cargo run --bin acorn -- export --schema --standard czml`.
- Handoff: Retire the plan if CZM-10 is dropped. Otherwise leave only CZM-10.

## CZM-10 — Type the AGI extension graphics

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: CZM-08, and a confirmed consumer. The extensions are in czml-writer `Schema/Extensions/AGI/` and are not rendered by open-source CesiumJS, which is unverified beyond the schema's `agi_` prefix.
- Exact write set: `crates/acorn-schema/src/standard/czml/graphics/agi.rs` (new), `crates/acorn-schema/src/standard/czml/graphics/mod.rs`, `crates/acorn-schema/src/standard/czml/packet.rs`, `crates/acorn-schema/src/standard/czml/property.rs` (`Direction`, `DirectionList`, `SensorVolumePortionToDisplay`), `crates/acorn-schema/src/standard/czml/enums.rs`, `crates/acorn-schema/src/standard/czml/tests/conformance.rs`, `tests/fixtures/schema/czml/schema-inventory.json`
- Primary integration lock: CZML graphics model (plan-local)
- Outcome: The six `agi_*` packet properties are typed and removed from the exclusion list. Their keys are no longer routed to `extensions`.
- Focused validation: `cargo nextest run --package acorn-schema czml::tests::conformance` with an empty exclusion list, and `make check-wasm`.
- Handoff: Retire the plan once CZM-14 is complete.
