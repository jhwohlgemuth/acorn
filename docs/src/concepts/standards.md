# Standards, formats, and interoperability

ACORN reads several metadata models and produces research records, linked data, archives, documents, and service messages. Support is specific to the part of a standard that ACORN uses. A successful local check does not certify an implementation, confirm that an identifier exists in a registry, or prove that a repository will accept a deposit.

## How to read this page

The terms below describe separate capabilities.

| Term | Meaning in ACORN |
| --- | --- |
| Parse | Decode a supported representation into ACORN's typed model. |
| Validate | Check modeled structure, field values, and cross-field rules without contacting an external certification authority. |
| Format | Parse and serialize a file canonically. Formatting performs additional value normalization only where this page says so. |
| Crosswalk | Map the fields for which ACORN defines a relationship between two metadata models. Warnings identify known losses. |
| Generate | Produce an artifact, schema, or protocol message in a supported representation. |
| Resolve | Contact an external registry or provider. Resolution is separate from local syntax validation and respects offline mode. |

Versions appear where ACORN pins one. An unversioned name describes the implemented compatibility surface, not every version or extension of that format.

## Metadata models

[`acorn check`](../cli-reference/commands/check.md) runs schema and semantic validation for the implemented models below. [`acorn format`](../cli-reference/commands/format.md) supports a smaller set. For CFF, DataCite, and InvenioRDM, formatting means parse and canonical reserialization. RAD formatting also normalizes controlled values and contact data, while ACRE formatting preserves comments and property order where its format permits.

| Model or profile | Accepted representations | `check` | `format` | Interchange and generation |
| --- | --- | --- | --- | --- |
| Research Activity Data (RAD) | JSON, JSONC, YAML, [ZON](zon.md), and ACORN Markdown; JSON Lines for batches | Structure, modeled fields, cross-field rules, quality, links, prose, and optional FAIR evidence | Canonical serialization plus controlled vocabulary, organization, affiliation, telephone, and contextual media normalization | RAD to DataCite, RAiD, CFF, JSON-LD, and document or archive exports; RAiD to RAD |
| Application Configuration for Research Enablement (ACRE) 1.0.0 | JSON, JSONC, and YAML | Schema validation | Two-space layout, LF line endings, and final newline; comments, property order, and existing trailing commas are retained where supported | JSON Schema generation |
| [Citation File Format 1.2.0](https://citation-file-format.github.io/1.2.0/) | `CITATION.cff`, YAML, and JSON | Structure, dates, identifiers, nested authors and references, prose, and links | Canonical serialization | Generated from RAD as `CITATION.cff`; no general CFF crosswalk target |
| [DataCite Metadata Schema 4.7](https://datacite-metadata-schema.readthedocs.io/en/4.7/) | JSON and YAML | Modeled structure, required fields, present recommended metadata, controlled values, identifiers, dates, rights, and relations | Canonical serialization | Direct conversion to DCAT, InvenioRDM, and HuWise; conversion from those models; RAD to DataCite |
| [CKAN package and Action API](https://docs.ckan.org/en/latest/api/) | JSON and YAML for direct checks; JSON, JSON Lines, YAML, and ZON in crosswalk batches | Modeled package, resource, organization, tag, temporal, and spatial fields | No | CKAN package to DCAT Dataset only |
| [Data Catalog Vocabulary 3](https://www.w3.org/TR/vocab-dcat-3/) | JSON and YAML | Modeled Dataset, Distribution, DataService, checksum, temporal, spatial, identifier, and relation fields | No | Direct conversion to DataCite; conversion to InvenioRDM and HuWise through DataCite |
| HuWise dataset metadata | JSON and YAML | Modeled dataset, contributor, funding, resource, rights, date, identifier, and relation fields | No | Direct conversion to DataCite; conversion to DCAT and InvenioRDM through DataCite |
| [InvenioRDM record metadata](https://inveniordm.docs.cern.ch/reference/metadata/) | JSON and YAML | Modeled record, creators, contributors, funding, rights, dates, identifiers, and CodeMeta fields | Canonical serialization | Direct conversion to DataCite; conversion to DCAT and HuWise through DataCite |
| RAiD metadata, governed by [ISO 23527:2022](https://www.iso.org/standard/75931.html) | JSON, JSONC, and YAML | Modeled RAiD metadata, identifiers, dates, roles, organizations, subjects, and related objects | No | RAD to RAiD and RAiD to RAD |
| [KitOps](https://kitops.org/) Kitfile and resolved [ModelKit](https://kitops.org/docs/modelkit/intro/) configuration | Authoring YAML and resolved JSON | Structure, manifest compatibility, paths, layer identities, digests, local package readiness, and referenced MCP Bundles | No general metadata formatting | Kitfile resolution and OCI ModelKit packaging |
| [Dublin Core](https://www.dublincore.org/specifications/dublin-core/dcmi-terms/) | In progress | In progress: recognized as a CLI standard name, but no analyzer is implemented | No | No generated schema or crosswalk yet |

Document and text selectors are analysis sources. They let `check` extract or analyze prose, but they are not metadata models.

### Schema generation

`acorn export --schema --standard <STANDARD>` prints JSON Schema for ACRE, RAD, CFF, CKAN, DataCite, DCAT, HuWise, InvenioRDM, RAiD, Kitfile, ModelKit, document, and text models. Dublin Core support is in progress and does not yet have a generated schema.

### Validation levels

ACORN keeps three questions separate:

1. Is the record valid for its own typed model?
2. Does a RAD contain the fields needed for the supported RAD to RAiD mapping?
3. Is a DataCite record deposit-ready under ACORN's required-property checks?

DataCite recommended and optional properties are validated when present. Their absence does not fail ordinary model validation. Crosswalk output should be checked against its target model before deposit or publication.

### Crosswalk boundaries

The structured crosswalk command accepts JSON, JSON Lines, YAML, and ZON. Its supported paths are:

```pikchr
scale = 0.82
ActivityBackground: box wid 1.65in ht 2.05in rad 0.14in fill 0xF2F2F2 thickness 0
HubBackground: box wid 1.85in ht 1.0in rad 0.14in fill 0xF2F2F2 thickness 0 at ActivityBackground+(2.25in,0.42in)
ExternalBackground: box wid 1.75in ht 4.05in rad 0.14in fill 0xF2F2F2 thickness 0 at HubBackground+(2.45in,-0.42in)
RAD: box "RAD" bold wid 1.15in ht 0.52in rad 0.08in fill 0xE8F1FB thickness 0 at ActivityBackground+(0,0.42in)
RAID: box "RAiD" bold wid 1.15in ht 0.52in rad 0.08in fill 0xE8F1FB thickness 0 at ActivityBackground+(0,-0.42in)
DataCite: box "DataCite" bold "crosswalk hub" italic wid 1.55in ht 0.62in rad 0.08in fill 0xFFF4D6 thickness 0 at HubBackground
CKAN: box "CKAN" bold wid 1.35in ht 0.52in rad 0.08in fill 0xE7F3E9 thickness 0 at ExternalBackground+(0,1.35in)
DCAT: box "DCAT" bold wid 1.35in ht 0.52in rad 0.08in fill 0xE7F3E9 thickness 0 at ExternalBackground+(0,0.45in)
Invenio: box "InvenioRDM" bold wid 1.35in ht 0.52in rad 0.08in fill 0xE7F3E9 thickness 0 at ExternalBackground+(0,-0.45in)
HuWise: box "HuWise" bold wid 1.35in ht 0.52in rad 0.08in fill 0xE7F3E9 thickness 0 at ExternalBackground+(0,-1.35in)
arrow from RAD.e to DataCite.w thick color 0x00662C
arrow <-> from RAD.s to RAID.n thick color 0x00662C
arrow <-> from DataCite.e to DCAT.w thick color 0x00662C
arrow <-> from DataCite.e to Invenio.w thick color 0x00662C
arrow <-> from DataCite.e to HuWise.w thick color 0x00662C
arrow from CKAN.s to DCAT.n thick color 0x00662C
```

Each arrow is a direct mapping. Conversions between DCAT, InvenioRDM, and HuWise follow two arrows through DataCite.

- RAD to DataCite or RAiD, and RAiD to RAD.
- CKAN to DCAT.
- DataCite directly to DCAT, InvenioRDM, or HuWise.
- DCAT, InvenioRDM, and HuWise directly to DataCite. Conversions between those three models use DataCite as the intermediate representation.

`--strict` rejects output when the conversion reports a warning. Warning coverage is strongest for direct conversions; loss reporting for routes through DataCite remains incomplete. CKAN conversion produces ACORN's flexible DCAT Dataset representation. It does not produce a DCAT Catalog or CatalogRecord and does not claim DCAT-US conformance.

## Field-level standards and profiles

Metadata models share validators from `acorn-schema`. Each schema chooses which rule applies to a field, so a rule listed here is not automatically applied to every string with a similar name.

### ISO and RFC representations

| Standard | ACORN enforcement |
| --- | --- |
| [ISO 8601](https://www.iso.org/iso-8601-date-and-time-format.html) | Calendar dates use `YYYY-MM-DD`. The reusable modern-date rule accepts years 1950 through 2029 and checks the supported month and day ranges. Year-only fields use four digits and, when a host clock is available, reject future years. ACORN does not implement the full set of ISO 8601 intervals, durations, reduced precision, or recurring dates. |
| [RFC 3339](https://www.rfc-editor.org/rfc/rfc3339) | Timestamp fields require a complete date and time with `Z` or a numeric UTC offset. Timestamp formatting normalizes accepted values to microsecond precision. |
| [ISO 3166-1](https://www.iso.org/iso-3166-country-codes.html) | Country fields that use the shared country rule must match an embedded alpha-2 code, case-insensitively. |
| [ISO 639:2023](https://www.iso.org/standard/74575.html) | DataCite and DCAT language fields use the alpha-2 codes historically published as ISO 639-1. InvenioRDM's general language-code rule accepts embedded alpha-2 or alpha-3 codes. Region subtags are outside these rules. |
| [ISO 2108:2017](https://www.iso.org/standard/65483.html) | ISBN fields use ACORN's ISBN-13 parser and check digit validation. ISBN-10 is not accepted. |
| [ISO/IEC 7064:2003](https://www.iso.org/standard/31531.html) | ISNI and ORCID use MOD 11-2 checksums. ROR uses its defined MOD 97-10 and Crockford Base32 representation. |
| [ISO 23527:2022](https://www.iso.org/standard/75931.html) | RAiD metadata and identifiers use the modeled RAiD representation. Local validation checks the identifier form and metadata rules; it does not query the RAiD service. |
| [ISO/IEC 18670:2025](https://www.iso.org/standard/89985.html) | SWHIDs use the canonical v1.2 identifier model supported by ACORN. Resolution is a separate network operation. |
| [RFC 6902](https://www.rfc-editor.org/rfc/rfc6902) | `acorn-diff` exposes all six JSON Patch operation shapes. Generated patches use deterministic key ordering and the add, remove, and replace subset. |
| [RFC 8493](https://www.rfc-editor.org/rfc/rfc8493) | BagIt import, download, and export validate the declaration, safe payload paths, complete manifests, and SHA-256 or SHA-512 checksums before publishing payload data. |

The year and date rules are intentionally different. A standalone year may represent older material, while the shared calendar-date rule is a narrower modern-date profile. Individual metadata models may add stricter range or chronology checks.

### Persistent identifiers

ACORN has local parsers for ARK, arXiv, DOI, Handle, ISBN, ISNI, ORCID, RAiD, ROR, RRID, SWHID, United States patent identifiers, and HTTP(S) URLs. Structured person and organization identifiers also check that known scheme names, scheme URIs, and identifier values agree.

These checks establish syntax, normalization, checksum, or scheme coherence. They do not establish ownership, registry status, publication status, or a relationship to the research activity. See [Persistent identifier lifecycle](persistent-identifiers.md) for each parser, canonical form, and resolver boundary.

### Other scalar profiles

Typed models also enforce required values, collection cardinality, string length, numeric range, nested validation, and whole-record conditions. Public metadata structs generally reject unknown fields where the external model has a closed shape. Open extension points retain their documented additional values.

| Field family | Enforced profile |
| --- | --- |
| URI and URL | General URI fields require parseable absolute references where their schema permits them. The stricter `uri` rule requires HTTP or HTTPS with a nonempty authority. This is the ACORN field profile, not a claim that every [RFC 3986](https://www.rfc-editor.org/rfc/rfc3986) production is accepted everywhere. |
| Email | The local part is nonempty, contains the supported ASCII character set, and is at most 64 characters. Domains use valid labels, IDNA conversion, or an IP literal. This is not a complete implementation of [RFC 5322](https://www.rfc-editor.org/rfc/rfc5322) or [RFC 6531](https://www.rfc-editor.org/rfc/rfc6531). |
| Telephone | The parser rejects known placeholder values and accepts ACORN's supported North American or international shape. Formatting emits the canonical dotted form where applicable. |
| UUID | UUID fields require the 36-character hyphenated shape. ACORN does not use this rule to certify every [RFC 9562](https://www.rfc-editor.org/rfc/rfc9562) version and variant bit. |
| [Semantic Versioning](https://semver.org/) | Version fields use the supported `major.minor.patch` shape. A leading `v` is accepted only in models that explicitly define it, such as Kitfile's `manifestVersion`. |
| [SPDX licenses](https://spdx.org/licenses/) | Fields documented as SPDX license identifiers currently require a nonempty value. ACORN can download SPDX license data for enrichment, but the shared schema rule does not prove list membership. |
| Media type and file format | Some schemas use controlled enums; other fields preserve nonempty media types, extensions, or format labels supplied by the source. A value's presence does not prove that its payload matches the declared media type. |
| Geospatial data | Latitude and longitude use numeric bounds. Polygon values must contain coordinate pairs and close the first and final point. These checks do not perform coordinate reference system transformation. |
| Source revisions and digests | Commit values contain 7 through 64 hexadecimal characters. Portable digest fields require a lowercase `sha256:` value with 64 hexadecimal characters; artifact workflows also compare declared sizes and digests with transferred bytes. |
| Network addresses | IPv4 and IPv6 fields use parsed address values. Email domain literals use the same address parsers. ACORN does not apply these parsers to ordinary host names. |
| Names and embedded values | Selected fields enforce camelCase, kebab-case, snake_case, numeric text, JSON syntax, ZON-safe text, or a restricted stable-identifier character set. These are field-specific application rules. |
| Regions and postal codes | The shared state or province rule accepts two ASCII letters. Postal codes accept a non-negative integer or up to 20 letters, digits, spaces, and hyphens. Neither rule verifies the value against a postal authority. |
| Image paths | Image fields that use the shared extension rule accept PNG, JPEG, GIF, WebP, TIFF, or SVG extensions. The extension check does not decode the file. |

### Vocabularies and linked data

RAD formatting resolves known keywords, technologies, sponsors, partners, organizations, and affiliations against ACORN's configured or embedded vocabularies. Metadata models also contain controlled enums for roles, resource types, date types, relation types, title types, and similar standard-specific values.

`acorn link` writes JSON-LD context for RAD using terms from [Schema.org](https://schema.org/), [Dublin Core Terms](https://www.dublincore.org/specifications/dublin-core/dcmi-terms/), [CodeMeta](https://codemeta.github.io/), DCAT, FOAF, BIBO, PROV-O, SKOS, and SPDX where the schema defines a mapping. A mapped term records the intended vocabulary relationship. It does not assert that every RAD record is a complete RDF graph for each vocabulary.

Contributor roles can use the [CRediT taxonomy](https://credit.niso.org/). Infrastructure metadata can record a referenced standard, its authority and version, the asserted relationship, and whether the claim is declared, inferred, or verified. Verified claims require evidence. OCP contribution, recognition, and management-profile records preserve authority data and test evidence; ACORN does not infer recognition or grant conformance.

## Product-wide formats and protocols

The metadata layer sits inside a wider set of interoperability boundaries.

### Serialization, documents, and archives

| Format | ACORN use and enforcement |
| --- | --- |
| [JSON](https://www.json.org/json-en.html) and [JSONC](https://jsonc.org) | Metadata, configuration, API, manifest, and linked-data representations. Typed readers reject malformed input and, for models that deny unknown fields, unknown properties. ACRE's JSONC formatter preserves comments and property order. |
| [YAML](https://yaml.org) | Metadata, ACRE configuration, CFF, and Kitfile authoring. Typed readers apply the selected schema after parsing. |
| [JSON Lines](https://jsonlines.org) | Batch RAD and crosswalk records. Each nonblank physical line must contain one complete JSON value; blank or malformed records fail with a record location. |
| [JSON-LD 1.1](https://www.w3.org/TR/json-ld11/) | Generated linked RAD with defined context mappings. ACORN generates its supported graph shape rather than acting as a general JSON-LD processor. |
| [JSON Schema](https://json-schema.org/specification) | Generated from ACORN's typed models for inspection and integration. The generated schema complements runtime semantic validation; it does not encode every cross-field or external check. |
| [ZON](zon.md) | ACORN's deterministic, strict representation of the JSON data model. ZON is an ACORN profile, not an ISO or RFC standard. |
| [Markdown](https://en.wikipedia.org/wiki/Markdown) | RAD authoring and export, prompt and skill content, and extracted document text. RAD Markdown requires ACORN's frontmatter discriminator or a recognized legacy representation. |
| PDF and OOXML | PDF, DOCX, and PPTX are read or generated by specific document workflows. ACORN validates the data and the parts it consumes or creates, but does not certify general PDF, WordprocessingML, or PresentationML conformance. |
| 7z, ZIP, TAR, and TAR.GZ | Containers for BagIt and other bounded workflows. Container parsing is separate from BagIt payload and checksum verification. |

### Research and model artifacts

| Format or specification | ACORN use and enforcement |
| --- | --- |
| [OCI Image Specification and Distribution Specification](https://specs.opencontainers.org/) | Model downloads parse schema version 2 manifests and indexes, validate media types, descriptor sizes, SHA-256 digests, and safe layer paths, then use ORAS with OCI Distribution-compatible registries. Runnable container images, ambiguous indexes, malformed descriptors, and unsafe paths fail closed. Digest integrity establishes content identity, not publisher authenticity. |
| KitOps ModelKit | ACORN implements the portable KitOps v1.15.0 Kitfile shape with `manifestVersion: v1.0.0`, authoring YAML, resolved JSON layer identities, and OCI packaging. Unsupported manifest versions produce compatibility warnings where the workflow can continue safely. |
| [CNCF ModelPack](https://github.com/cncf/modelpack-spec) | Model downloads recognize the implemented ModelPack manifest, configuration, role, and layer media types for weights, weight configuration, documentation, code, and datasets. Raw, TAR, gzip, and Zstandard layer forms are bounded and path-checked. |
| [GGUF](https://github.com/ggml-org/ggml/blob/master/docs/gguf.md) | ACORN checks the magic value, supported container version, bounded metadata, tensor declarations and data bounds, and split-file consistency before using a model. This is a safe-consumption check, not validation of every model architecture or tensor semantic. |
| [MCP Bundle](../packages/mcpb.md) | Generated and inspected `.mcpb` archives follow manifest version 0.3 at ACORN's pinned upstream revision. ACORN checks the root manifest, required identity and server fields, safe archive paths, in-archive entry point, and SHA-256 identity where used. Bundle creation and inspection never execute the server. |

### Service and agent protocols

| Protocol or profile | ACORN boundary |
| --- | --- |
| [Model Context Protocol](https://modelcontextprotocol.io/specification/) | ACORN provides a local stdio server and an outbound client for configured stdio and Streamable HTTP servers. Native initialization, tool discovery, JSON Schema input, calls, cancellation, and shutdown remain MCP messages. Policy controls offline and mutation capabilities. |
| [Agent Client Protocol](https://agentclientprotocol.com/get-started/architecture) v1 | The ACP client supervises one-shot external-agent sessions, readiness and capability negotiation, streamed updates, model selection, cancellation, and fixed-policy permission responses. ACP is the default inference backend for bounded fix suggestions. |
| [JSON-RPC 2.0](https://www.jsonrpc.org/specification) | The HTTP endpoint requires JSON-RPC 2.0 request envelopes and validates method parameters against the shared operation registry before dispatch. Authentication, body limits, offline policy, and mutation policy are transport controls around the JSON-RPC contract. |
| [Protocol Buffers](https://protobuf.dev/) and [gRPC](https://grpc.io/) | The optional `acorn.rpc.v1.AcornRpc` contract exposes the shared operation registry through generated Protobuf messages over gRPC. It enforces authentication, message limits, deadlines, HTTP/2, and TLS for non-loopback endpoints. |
| [OpenAPI 3.1](https://spec.openapis.org/oas/v3.1.0) | The importer reads YAML or JSON descriptions with a `paths` object and creates ACORN resource entries. It is a focused import adapter, not a complete OpenAPI validator or client generator. |
| OpenAI-compatible HTTP APIs | Completion, chat-completion, response, embedding, image, and audio operations use the implemented OpenAI-style JSON request and response shapes. Compatibility describes an API profile and does not imply that a server is operated by OpenAI or implements every OpenAI endpoint. |
| [Dotprompt](https://github.com/google/dotprompt/tree/main/spec) and [Agent Skills](https://agentskills.io/specification) | ACORN embeds `.prompt` templates and distributes `SKILL.md` packages for compatible consumers. These files carry versioned instructions and resources; their presence does not grant tools or permissions. |

MCP[^mcp], ACP[^acp], [JSON-RPC](https://www.jsonrpc.org), and [gRPC](https://grpc.io) share selected ACORN operations and policy metadata, but each keeps its native framing, discovery, session, and error semantics. Compatibility with one transport does not imply wire compatibility with another.

## Where ACORN enforces a rule

Use the command that matches the question:

```sh
# Validate a record against one metadata model
acorn check metadata.json --standard datacite

# Preview canonical formatting without modifying the file
acorn format index.json --dry-run

# Convert metadata and fail if a known loss is reported
acorn export metadata.json --from datacite --to dcat --format json --strict --dry-run

# Inspect the generated schema for a model
acorn export --schema --standard raid

# Generate linked RAD as JSON-LD
acorn link index.json --dry-run
```

Network-backed resolution and enrichment are separate operations. Use global `--offline` when validation must remain local and reproducible.

[^mcp]: Model Context Protocol, ["Architecture"](https://modelcontextprotocol.io/specification/2025-06-18/architecture) and ["Server features"](https://modelcontextprotocol.io/specification/2025-06-18/server/index).
[^acp]: Agent Client Protocol, ["Architecture"](https://agentclientprotocol.com/get-started/architecture).
