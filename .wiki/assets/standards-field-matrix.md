---
okf_version: "0.1"
---

# Standards Field Matrix — RAD | RAiD 1.x | DataCite 4.7 | DCAT 3 | CodeMeta | Schema.org

**Pin:** DataCite kernel-4.7 `https://schema.datacite.org/meta/kernel-4.7/metadata.xsd` + JSON API, RAiD `https://metadata.raid.org/en/latest/` (ISO 23527:2022), DCAT 3 `https://www.w3.org/TR/vocab-dcat-3/`, CodeMeta `https://codemeta.github.io/`, Schema.org `https://schema.org/`. Reconciled `2026-09-17` against `acorn-schema` at `0.2.0` after completion of the RAD alignment implementation.

Slice 0 fixtures (referenced per row): `tests/fixtures/schema/datacite/4.7/valid-4.7-deltas.json` (Poster/Presentation + RAiD/SWHID/Other+info + RelatedItem), `tests/fixtures/schema/datacite/4.7/valid-4.7-related-item.xml` (Resource + RelatedItems), `tests/fixtures/schema/datacite/4.7/invalid-missing-mandatory.json` (missing `titles`), `tests/fixtures/schema/datacite/4.7/invalid-missing-creators.json`.

## Validity Levels

| Level | Gate | Mandatory Fields | Source |
|---|---|---|---|
| Valid RAD | `Validate` derive | `ResearchActivity.title` len 4..MAX, `ResearchActivityMetadata.identifier` kebabcase, `ContactPoint`, `Sections`, vocab-normalized keywords/technology | `crates/acorn-schema/src/research_activity/mod.rs:160`, `crates/acorn-schema/src/validation/` |
| RAiD-ready | `pid/raid/mod.rs` `Metadata::validate` | `access` (required), `contributor` ≥1 (leader/contact flags), `date.startDate`, `title` Primary≥1, `organization` LeadResearchOrganisation | `crates/acorn-schema/src/pid/raid/mod.rs` |
| Deposit-ready | `validate_deposit_ready(&Record)` / `&Resource` | `doi`/`identifier`, `titles`, `creators`, `publisher`, `publicationYear`, `resourceTypes`/`resourceType` (general) | `crates/acorn-schema/src/standard/datacite.rs:1525` (distinct from `Validate`) |

Recommended vs optional: `subjects`, `contributors`, `dates`, `relatedIdentifiers`, `descriptions`, `geoLocations`; Optional: `language`, `alternateIdentifiers`, `sizes`, `formats`, `version`, `rightsList`, `fundingReferences`, `relatedItems` (DataCite 4.7 prop 20) — validated but not deposit-blocking.

## Matrix — RAD Authoring to Standards

Columns: `Classification`: `exact` (reversible, no loss) | `transformed` (type/syntax coercion) | `lossy` (semantic narrowing, warn) | `local-only` (no target, warn if projected) | `conditionally-required` (deposit gate) | `unsupported` (explicit gap).

Rows use serialized keys (JSON) where relevant, occurrence `0..1`/`1`/`0..n`, vocabulary owner, normalization.

### ResearchActivity (top-level `research_activity/mod.rs:101`, `deny_unknown_fields`)

| RAD Field | Changelog | ResearchOutput | RAiD Path (occ, vocab) | DataCite JSON `Attributes` | DataCite XML `Resource` | DCAT 3 | CodeMeta | Schema.org | Classification | Transform | Vocabulary Owner | Occ | Normalization |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| `@context` (`Option<ResearchActivityContext>`) `compat` | — | — | `—` | `—` | `—` | `—` | `—` | `https://www.w3.org/TR/json-ld11/#the-context` | local-only | — | W3C | 0..1 | JSON-LD CURIE via `namespaces.rs` | `research_activity/tests/` |
| `@type` (`Option<String>`) | — | — | `—` | `—` | `—` | `—` | `—` | `schema:CreativeWork` | local-only | — | Schema.org | 0..1 | free text | — |
| `meta` (`ResearchActivityMetadata`, `nested`) | — | — | `—` (container) | `—` | `—` | `—` | `—` | `—` | exact | — | ACORN | 1 | nested `Validate` | — |
| `aspect` (`Option<AspectFramework>`) `compat` | — | — | `—` | `—` | `—` | `—` | `—` | `—` | local-only | — | ACORN ASPECT | 0..1 | ASPECT framework | `aspect/` |
| `title` (`String`, len 4..MAX, `string_trim`, default) | `summary` (local-only) | `title` | `title[]` `TitleType=Primary` `1..n` | `titles[0].title` | `titles/title` | `dcterms:title` | `name` | `schema:name` | transformed | trim | ACORN → RAiD/DataCite/DCAT | 1 | `string_trim` | `datacite.json: Example Dataset Title` |
| `subtitle` (`Option<String>`, max 75, `option_string_trim`) | `summary` (local-only) | — | `title[]` `AlternativeTitle` | `titles[+1].title` `titleType=AlternativeTitle` | `titles/title @titleType=AlternativeTitle` | `dcterms:alternative` | — | `schema:alternativeHeadline` | transformed | trim | ACORN | 0..1 | `option_string_trim` | — |
| `sections` (`Option<Sections>`, `nested`, `compat`; required when `draft=false`) | `entries[].summary` (local-only) | — | `description[]` (mapped aggregate, not per-entry) | `descriptions[]` `Abstract/Methods/...` (aggregate only) | `descriptions/description` | `dcterms:description` | — | `schema:description` | lossy | DataCite Project export flattens authored sections into typed descriptions with a `Simplified` warning; logbook entries remain local-only and each emits a warning | ACORN | 0..1 draft, 1 non-draft | prose | `datacite.xml: descriptions[0]=Abstract` |
| `contact` (`ContactPoint`, `nested`, `compat`) | — | — | `contributor[]` (`leader`/`contact`) | `creators[0]` / `contributors` | `creators/creator` | `dcat:contactPoint` | `author` | `schema:author` | transformed | `ContactPoint` → `Contributor`/`Creator` | ORCID/ROR | 1 | `format_phone_number` etc | `raid` org |
| `notes` (`Option<Other>`) `compat` | — | — | `—` | `—` | `—` | `—` | `—` | `schema:Text` | local-only | — | ACORN | 0..1 | free text | — |
| `logbook` (`Option<Logbook>`) | `entries[]` (`identifier` kebab-unique, `timestamp` RFC3339 offset, `category` Administration/Collaboration/Data/Engagement/Funding/Infrastructure/Method/Other/Personnel/Publication/Research/Software with read aliases Code/Conference/General/Technical/Tour, optional `classification`, optional `eventType` Conference/Exhibition/Meeting/Other/Presentation/Tour/Training/Workshop, `state` Completed/InProgress/Upcoming, `origin` Manual/Discovered + `discovery` provenance, `summary` nonempty≤500, `items` 1..20; future `references: Vec<TypedRelatedResource>` for nanopub URIs, reserved) | — (refs `ResearchOutput` via typed related resource; future `references` holds nanopub Trusty URIs `http://purl.org/np/…`, not embedded assertion) | `—` (do not map `timestamp` to RAiD interval/metadata timestamps; `category/eventType/state/summary/items/origin` + future `references` local-only until DataCite `relatedIdentifierType=NANOPUB` approved) | `—` (do not map `timestamp` to `dateType=Updated`; aggregate `summary→Description Other` opt-in, per-entry `local-only` warn; future `references` → `relatedIdentifiers` with `References`) | `—` | `—` | `—` | Closest published ontology classes for `category` and `eventType` from SKOS, PROV-O, DCAT, Schema.org, and BIBO; no ACORN-owned or invented ontology IRIs | local-only | preserve order, require offset, stable `identifier`; `since`-filtered `Target::Updates` export (PDF/MD/DOCX) separate from RAD | ACORN + SKOS + PROV-O + DCAT + Schema.org + BIBO | 0..1 (`entries` 1..1000) | `timestamp` built-in, `nonempty`; classification entries require Administration/Other and activity classification must dominate; archive-not-delete graduation to `meta`/`contact` | `logbook` whiteboard + `since` filtered `updates` fixture |

### ResearchActivityMetadata (`research_activity/mod.rs`, `rename_all=camelCase`, 35 fields)

Every `Option<OneOrMany<T>>` field below accepts one `T` or an array at input. Scalars normalize to one-item arrays; omitted and `null` values normalize to `None`; and explicit empty arrays remain `Some([])`. Canonical output uses arrays.

| RAD Field | RAiD Path | DataCite JSON | DataCite XML | DCAT 3 | CodeMeta | Schema.org | Classification | Transform | Vocab | Occ | Normalization | Fixture |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| `@context` / `@type` | `—` | `—` | `—` | `—` | `—` | JSON-LD | local-only | — | W3C | 0..1 | CURIE | — |
| `enrichment: Option<Vec<EnrichmentProvenance>>` (`provider`, `source`, `fields`) | `—` | `—` | `—` | `—` | `—` | provenance | local-only | — | ACORN `Provider` (Orcid/Ror/CiteAs/OpenAlex/Osti) | 0..n | `kebabcase` fields | — |
| `classification: Option<ClassificationLevel>` (Unclassified/Confidential/Secret/TopSecret) | `—` | `—` | `—` | `—` | `—` | `schema:DefinedTerm` | local-only | — | ACORN | 0..1 | enum | — |
| `archive: bool` (input optional, default false), `draft: bool` (input optional, default true) | `—` | `—` | `—` | `—` | `—` | `schema:Boolean` | local-only | canonical output emits both values | ACORN | 0..1 input, 1 output | bool | — |
| `status: Status` (Active/Paused/Completed/Canceled, default Active) | `—` (distinct from dated `contributor`/`organization` roles and `date` interval; also distinct from `LogbookState`) | `—` | `—` | `—` | `—` | `schema:DefinedTerm` | local-only | — | ACORN | 1 | enum | — |
| `identifier: String` (`identifier`, alias `id`, `kebabcase`, default) | `identifier` (`MetadataIdentifier`) `0..1` | `—` (activity DOI via Project export `ResourceTypeGeneral::Project` `datacite.rs:407`, opt-in) | `—` | `dcterms:identifier` | `identifier` | `schema:identifier` | exact | kebabcase + trim | ACORN local | 1 | `string_trim` | `raid` id |
| `doi: Option<Vec<String>>` (`is_attribute_publication_identifier_list` DOI\|arXiv) | `relatedObject[]` `DOI` | `relatedIdentifiers[]` `DOI` / `relatedItems` | `relatedIdentifiers/relatedIdentifier @relatedIdentifierType=DOI` | `dcat:relation` | — | `bibo:doi` | transformed | DOI/arXiv parse | DOI Foundation/arXiv | 0..n | `PublicationIdentifierType` | `datacite.json` DOI |
| `handle: Option<Vec<String>>` (`is_attribute_handle_list`) | `alternateIdentifier` | `relatedIdentifiers[]` `Handle` / `alternateIdentifiers` | `alternateIdentifiers` | `dcterms:identifier` | — | `schema:identifier` | transformed | `Handle::format` | Handle | 0..n | `Handle::is_valid` | — |
| `books: Option<Vec<String>>` (`is_attribute_books` ISBN) | `relatedObject` | `relatedIdentifiers[]` `ISBN` | `relatedIdentifiers` | `dcat:relation` | — | `schema:isbn` | transformed | ISBN | ISBN | 0..n | `is_isbn` | — |
| `patents: Option<Vec<String>>` | `relatedObject` | `relatedIdentifiers[]` `URN`/`Patent` | `relatedIdentifiers` | `dcat:relation` | — | — | transformed | free | — | 0..n | string | — |
| `publications: Option<Vec<String>>` (`each(url)`) | `relatedObject` (URL) | `relatedIdentifiers[]` `URL` | `relatedIdentifiers` | `dcterms:relation` | — | `schema:url` | transformed | URL | — | 0..n | `url` | — |
| `outputs: Option<Vec<ResearchOutput>>` (`nested`) | `relatedObject` (PID-bearing outputs) | `relatedIdentifiers` + `relatedItems` (output metadata) | `relatedIdentifiers`/`relatedItems` | `dcat:distribution`/`qualifiedRelation` | `CodeMeta` (software) | `schema:CreativeWork` | exact (canonical) + transformed (crosswalk) | `ResearchOutput` ↔ `Record`/`RelatedItem` | DataCite/Crossref/ROR | 0..n | `Validate` nested | `valid-4.7-deltas.json: outputs via relatedItems` |
| `raid: Option<Vec<String>>` (`each(doi)`) | `identifier` / `alternateIdentifier` `RAiD` `10.26259/...` | `relatedIdentifiers[]` `RAiD` `datacite.rs:168` | `relatedIdentifiers @RAiD` | `dcterms:identifier` | — | `schema:identifier` | exact | RAiD `10.83962/` + `https://raid.org/` | ARDC ISO 23527 | 0..n | `doi` | `valid-4.7-deltas.json: relatedIdentifiers RAiD` |
| `swhid: Option<Vec<String>>` (`is_attribute_swhid_list` `swh:1:...`) | `—` | `relatedIdentifiers[]` `SWHID` `datacite.rs:168` | `relatedIdentifiers @SWHID` | `dcat:identifier` | — | `codemeta:identifier` | exact | SWHID v1.2 `swh:1:cnt:` etc | Software Heritage ISO 18670 | 0..n | `SWHID::is_valid` | `valid-4.7-deltas.json: SWHID` |
| `ror: Option<Vec<String>>` (`each(ror)`) | `organization[].id` `https://ror.org/...` | `creators[].affiliation` `ROR` / `affiliationIdentifier` | `affiliation` | `dcterms:publisher` / `foaf:Organization` | — | `schema:affiliation` | transformed | ROR `https://ror.org/` | ROR | 0..n | `ror` | — |
| `additional_type: Option<OrganizationType>` | `—` | `—` | `—` | `—` | — | `schema:additionalType` | local-only | — | ACORN | 0..1 | enum | — |
| `media: Option<Vec<MediaObject>>` (`graphics` alias, `nested`) | `—` | `—` | `—` | `dcat:mediaType` | — | `schema:MediaObject` | local-only | — | — | 0..n | `has_image_extension` | — |
| `websites: Option<Vec<Website>>` (`nested`) | `alternateUrl` | `relatedIdentifiers[]` `URL` (`IsSupplementTo` etc) | `relatedIdentifiers` | `dcat:landingPage` / `distribution.accessURL` | `url` | `schema:url` | transformed | URL | — | 0..n | `url` | `datacite.json: url` |
| `keywords: Vec<Keyword>` (default `[]`, omitted when empty, `OneOrMany` input, lower-kebab-case, `ControlledVocabulary`) | `subject[]` `keyword` `0..n` | `subjects[]` `subject` `datacite.rs:1100` | `subjects/subject` | `dcat:keyword` | `keywords` | `schema:keywords` | exact | `ControlledVocabulary::normalize` | ACORN `keywords.csv` | 0..n | trim, normalize scalar to array, lower-kebab | `datacite.json: Climate change` |
| `technology: Vec<String>` (default `[]`, omitted when empty, `OneOrMany` input, `ControlledVocabulary`) | `—` | `—` | `—` | `—` | `programmingLanguage` | `schema:DefinedTerm` | local-only | normalize via `technology.csv` | ACORN | 0..n | trim, normalize scalar to array | — |
| `sponsors: Option<Vec<String>>` | `organization[]` `Funder` role | `fundingReferences[]` `funderName`/`funderIdentifier` `datacite.rs:733` | `fundingReferences/funderName` | `dcterms:publisher` | `funding` | `schema:sponsor` | transformed | `Vocabulary::Sponsors` via `sponsors.csv`, precedence vs structured `Funding` `output.rs:50` | ROR/Crossref | 0..n | `ControlledVocabulary` | — |
| `partners: Option<Vec<String>>` | `organization[]` `LeadResearchOrganisation` | `contributors[]` `HostInstitution` | `contributors` | `dcterms:contributor` | — | `schema:contributor` | transformed | `partners.csv` | — | 0..n | `ControlledVocabulary` | — |
| `related: Option<Vec<String>>` (related RAD `identifier` k-b) | `relatedRaid[]` / `relatedObject` `category` | `relatedIdentifiers[]` `IsPartOf`/`HasPart` | `relatedIdentifiers` | `dcterms:relation` | — | `schema:isPartOf` | transformed | kebabcase id → `10.26259/`? | ACORN | 0..n | `kebabcase` | — |
| `resources: Option<OneOrMany<Resource>>` (`hardware::Resource` ASIC/CPU/GPU…) | `—` (minimum compute requirement, distinct from configured infrastructure) | `—` (not `resourceType`) | `—` | `—` | — | `schema:DefinedTerm` | local-only | existing manual `Resource::validate` runs through nested RAD validation | ACORN hardware | 0..n | scalar-or-array input, canonical array | `hardware` |
| `infrastructure: Option<OneOrMany<ComputeEnvironment>>` | `—` | `—` | `—` | `—` | — | `dcterms:relation` | local-only | RAiD and DataCite project exports emit `NoEquivalent` warnings; environment composes platforms, optional facility with operator ROR identifiers, standards claims, OCP records, and management-profile evidence | ACORN + OCP + ROR | 0..n | nested validation, scalar-or-array input, canonical array | `schema/ocp/infrastructure.json` |

### ResearchOutput (`research_activity/output.rs:110`, 26 fields, `rename_all=camelCase`)

| RAD Field | DataCite JSON | DataCite XML | RAiD | DCAT 3 | CodeMeta | Classification | Transform | Vocab | Occ | Notes |
|---|---|---|---|---|---|---|---|---|---|---|
| `identifier: Option<String>` (`option_string_trim`) | `alternateIdentifiers[]` (`alternateIdentifierType=ACORN Provider`) | `alternateIdentifiers` | `relatedObject` | `dcterms:identifier` | `identifier` | exact | trim plus reserved adapter type | ACORN | 0..1 | Provider-native identifier remains distinct from typed `alternateIdentifiers`. |
| `doi: Option<String>` (`doi`) | `doi` (prop 1) `datacite.rs:515` | `identifier` `DOI` `datacite.rs:1128` | `relatedObject DOI` | `dcat:identifier` | `identifier` | conditionally-required | DOI | DOI | 0..1 | deposit: required via `validate_deposit_ready` |
| `title: String` (`length(min=1)`, `string_trim`) REQUIRED | `titles[0].title` (prop 3) | `titles/title` | `title Primary` | `dcterms:title` | `name` | exact | trim | — | 1 | `datacite.json: Example Dataset Title` |
| `type` (`kind: Option<String>`, `rename="type"`) | provider-specific input only | provider-specific input only | `relatedObject type` | `dcterms:type` | `type` | local-only | trim | provider vocabulary | 0..1 | Distinct from the controlled DataCite resource type fields. |
| `publicationYear: Option<i32>` (`year`) | `publicationYear` | `publicationYear` | related-object date when representable | `dcterms:issued` | `datePublished` | transformed | year | DataCite year | 0..1 | DataCite-deposit mandatory. |
| `dates: Option<Vec<TypedActivityDate>>` | `dates[]` | `dates/date` | activity/output date projection | `dcterms:issued`/temporal | date properties | transformed | typed RFC 3339 timestamp | DataCite `DateType` | 0..n | `Issued` may supply publication context without replacing `publicationYear`. |
| `publisher: Option<datacite::Publisher>` | `publisher` | `publisher` | organization relation | `dcterms:publisher` | `publisher` | exact | direct DataCite shape | name plus optional identifier | 0..1 | DataCite-deposit mandatory. |
| `resourceTypeGeneral: Option<ResourceTypeGeneral>` | `types.resourceTypeGeneral` | `resourceType @resourceTypeGeneral` | `relatedObject.type` | `dcterms:type` IRI | `type` | transformed | controlled mapping | DataCite 4.7 | 0..1 | Includes `Poster` and `Presentation`. |
| `resourceType: Option<String>` | `types.resourceType` | `resourceType` text | related-object category where representable | `dcterms:type` label | `additionalType` | transformed | trim | free text | 0..1 | Specific type complements `resourceTypeGeneral`. |
| `language: Option<String>` | `language` | `language` | language | `dcterms:language` | `inLanguage` | exact | ISO 639-1 validation | ISO 639-1 | 0..1 | Portable validation only. |
| `alternateIdentifiers: Option<Vec<TypedIdentifier>>` | `alternateIdentifiers[]` | `alternateIdentifiers` | `alternateIdentifier[]` | `dcterms:identifier` | `identifier` | transformed | typed identifier mapping | extensible scheme | 0..n | Scheme and optional scheme URI are retained. |
| `relatedIdentifiers: Option<Vec<TypedRelatedResource>>` | `relatedIdentifiers[]` | `relatedIdentifiers` | `relatedObject[]`/`relatedRaid[]` | qualified relation | `citation`/`isPartOf`/related properties | transformed | relation vocabulary mapping | DOI/RAiD/RRID/SWHID/etc. | 0..n | The RRID leaf crosswalk canonicalizes the identifier and preserves explicit DataCite relation, relation information, general resource type, and scheme URI. It reports the local category and DataCite-only metadata-scheme fields as structured `NoEquivalent` losses. Record assembly, `Other` relation information, and whole-activity Project assembly are implemented. JATS 1.4 RRID markup is pinned in `tests/fixtures/rrid/jats-1.4.xml` without claiming a general XML adapter. |
| `relatedItems: Option<Vec<datacite::RelatedItem>>` | `relatedItems[]` | `relatedItems` | related object when representable | qualified relation | related creative work | transformed | direct DataCite shape then target mapping | DataCite 4.7 | 0..n | Distinct from identifier-only relations. |
| `version: Option<String>` (`version`) | `version` | `version` | alternate identifier/context | `dcat:version` | `version` | exact | semantic-version validation | SemVer | 0..1 | Non-semantic provider versions remain unsupported. |
| `rightsList: Option<Vec<StructuredRights>>` | `rightsList[]` | `rightsList/rights` | `access`/license | `dcterms:license`/`accessRights` | `license` | transformed | SPDX and URI mapping | SPDX/extensible | 0..n | Canonical structured rights; distinct from OA summary `access`. |
| `descriptions: Option<Vec<Description>>` | `descriptions[]` | `descriptions/description` | descriptions | `dcterms:description` | `description` | transformed | typed description mapping | DataCite `DescriptionType` | 0..n | Direct DataCite type reuse. |
| `subjects: Option<Vec<ControlledSubject>>` | `subjects[]` | `subjects/subject` | `subject[]` | `dcat:theme`/`keyword` | `keywords` | transformed | scheme/value URI mapping | extensible scheme | 0..n | Complements authoring-friendly `keywords`. |
| `formats: Option<Vec<String>>` | `formats[]` | `formats/format` | related-object metadata | `dcterms:format` | `encodingFormat` | exact | preserve value | MIME/extension | 0..n | Adapter decides whether a value is a MIME type or free text. |
| `sizes: Option<Vec<String>>` | `sizes[]` | `sizes/size` | related-object metadata | distribution metadata | `contentSize` | transformed | preserve DataCite text | free text | 0..n | Examples include page and byte counts. |
| `geoLocations: Option<Vec<GeoLocation>>` | `geoLocations[]` | `geoLocations` | `spatialCoverage[]` | `dcterms:spatial` | `spatialCoverage` | transformed | point/box/polygon mapping | DataCite geometry | 0..n | Direct DataCite type reuse. |
| `creators: Option<Vec<Contributor>>` | `creators[]` | `creators/creator` | `contributor[]` | `dcterms:creator` | `author` | transformed | person or organization plus affiliation mapping | ORCID/ROR | 0..n | `nameType` distinguishes personal and organizational creators; organization ROR identifiers round-trip through `nameIdentifiers`. |
| `contributors: Option<Vec<Contributor>>` | `contributors[]` | `contributors/contributor` | `contributor[]` | `dcterms:contributor` | `contributor` | transformed | person or organization plus DataCite contributor type | DataCite/CRediT/ORCID/ROR | 0..n | DataCite `contributorType` is exact. CRediT `roles[]` remain canonical RAD metadata and emit `NoEquivalent` on DataCite projection because DataCite 4.7 has no CRediT role property. |
| `funding: Option<Vec<Funding>>` (`name`, `identifier`, `identifierType`, `schemeUri`, `ror`, `awards`) | `fundingReferences[]` | `fundingReferences[]` | funder organization | `dcat:qualifiedRelation` | `funding` | transformed | typed funder registry and award mapping; grant DOI normalizes through `awardURI` | ROR/Crossref/GRID/ISNI/Other | 0..n | General DataCite funder identifier types and compatibility ROR values round-trip; conflicting identifiers fail conversion. |
| `websites: Option<Vec<Website>>` | `url`/related identifier | related identifier URL | `alternateUrl` | `dcat:landingPage`/distribution | `url`/`codeRepository` | transformed | URL mapping | URL | 0..n | Website kind selects the target property. |
| `keywords: Vec<Keyword>` | `subjects[]` | `subjects/subject` | `subject` | `dcat:keyword` | `keywords` | exact | `Keyword` normalize | ACORN `keywords.csv` | 0..n | `datacite.json: subjects[0] Climate change` |
| `access: Option<Access>` (`open`, `status`, `url`, `license`) | derived access summary only | derived access summary only | `access` | `dcterms:accessRights` | access mode | local-only | provider OA summary | provider vocabulary | 0..1 | Structured license exchange uses `rightsList`. |

**Profiles completed 2026-09-17:** `ResearchOutput <-> DataCite Record`, output-specific deposit readiness, organizational agents, DataCite contributor types, CRediT role storage, complete funder identifier schemes, whole-activity RAiD conversion, strict DataCite Project export, and structured conversion-loss reporting are implemented.

### RAiD Top-Level (`pid/raid/mod.rs` `Metadata`, `deny_unknown_fields`, `Validate`)

| RAiD Field | RAD Source | Classification | Transform | Occ | Vocabulary |
|---|---|---|---|---|---|
| `identifier` (`MetadataIdentifier`) | `meta.raid` plus explicit registry metadata | conditionally-required | normalize the authored RAiD; require `RaidExportOptions::identifier` in strict mode or explicitly infer registry fields | 0..1 | DOI-like |
| `title[]` (`TitleType` Primary required) | `title` + `subtitle` | transformed | `subtitle` → `AlternativeTitle` | 1..n | — |
| `contributor[]` (`id`, `role`, `position`, `leader`, `contact`, `email`) | `contributors[]` plus legacy primary `contact` | transformed | map typed identifiers, affiliations, dated roles, and assertions without inventing flags | 1..n | ORCID/ISNI, CreditRole |
| `organization[]` (`id`, `role`) | `organizations[]`, `funders[]`, plus legacy `ror`/`partners` | transformed | map typed identifiers and dated roles; funders map separately | 0..n | ROR, organization roles |
| `date` (`startDate` required) | `typedDates[]` | transformed | select typed start/end values; never infer current time | 1 | timestamp/date type |
| `relatedObject[]` (`id`, `schemaUri`, `type`, `category`) | `relatedResources[]` and PID-bearing `outputs[]` | transformed | typed identifier and relation mapping | 0..n | `RelatedObjectType` |
| `relatedRaid[]` | `relatedResources[]` with RAiD identifiers | transformed | normalize RAiD identifier and relation | 0..n | `RelatedRaidType` |
| `subject[]` (`keyword`) | `subjects[]` plus authoring `keywords` | transformed | preserve controlled scheme/value URI where supported | 0..n | RAiD subject/ACORN keywords |
| `spatialCoverage[]` (`place`) | `spatialCoverage[]` | transformed | map structured place records | 0..n | `Place` |
| `access` | `access` and `rights[]` | transformed | map structured access and rights assertions | 1 | RAiD access vocabularies |
| `description` / `alternateIdentifier` / `alternateUrl` / `traditionalKnowledgeLabel` | prose, identifier, and website fields where approved | transformed or unsupported | adapter must report omissions | 0..n | target-specific |
| `metadata` (`created`/`updated`) | `typedDates[]` with `Created`/`Updated` | transformed | select explicit values only; do not use logbook timestamps | 0..1 | timestamp/date type |

Inference rule: **Do not fabricate** `date`, `leader`/`contact` role, `owner`/`registrationAgency`, `CC0` license, or `relatedObject` when RAD absent. The implemented adapter gates supported defaults behind `RaidExportOptions::allow_infer`, which defaults to false, requires an explicit `inference_date`, and emits `ConversionWarning::Approximated` for every inferred assertion.

### DCAT 3 / Invenio / HuWise / CFF Crosswalk (existing `crosswalk/`)

| Concern | Source Field | Target Field | Mapping | Classification | Vocab |
|---|---|---|---|---|---|
| Identity | `doi` (`String`) `datacite.rs:168` | `identifier` (`StringVec`) `dcat` | `as_vec` `mapping.rs:482` (`Poster` etc) | transformed | DOI ↔ StringVec |
| Publication year | `publication-year` (`Number`) | `issued` (`Date`) | `year_to_date` / `extract_year` `mapping.rs:467` | transformed | year ↔ `2024-01-01` |
| Language | `language` `String` | `language` `StringVec` | `as_vec` / `first_of_vec` `mapping.rs:440` | transformed | ISO 639-1 `is_iso_639_1_language_code` |
| License | `license` `IRI` | `rightsURI` | direct | exact | SPDX `validate_license` |
| Type | `resource-type` `String` | `type_` `IRI` | `resource_type_to_iri` / `iri_to_resource_type` `mapping.rs:482` (`Poster`→`Poster`) | transformed | DataCite ↔ DCAT IRI |

`Standard::crosswalk` supports `datacite ↔ dcat ↔ invenio ↔ huwise`, `rads -> raid`, `raid -> rads` with required caller identity enforced, and opt-in `rads -> datacite` Project export. Structured CLI export infers RAD and RAiD documents, reports conversion warnings during dry runs, and makes strict mode fail before writing. The DataCite Project profile requires exactly one authored project DOI, one `Issued` date, and one organization with a `Publisher` role; it does not infer deposit assertions.

## Controlled Vocabularies & Normalization

* **Identifiers:** DOI `doi` `https://doi.org/10.5072/...`, RAiD `10.83962/...` `https://raid.org/10.26259/...`, SWHID `swh:1:cnt:...` `is_attribute_swhid_list`, ROR `https://ror.org/...` `ror`, ORCID `https://orcid.org/...` `orcid`, ISNI `https://isni.org/isni/...` `validate_name_identifier`, Handle `hdl:…` `Handle::is_valid`.
* **Languages:** `is_iso_639_1_language_code` `datacite.rs:516` (e.g. `en`), DCAT `language` as `StringVec` via `as_vec`.
* **Dates:** DataCite `Date.timestamp` RFC3339 offset `Validate(timestamp)`, `publicationYear` `year`, RAiD `unix_epoch`/`date` with `has_start_date`; do not map `Logbook.timestamp` to `Updated`; `Target::Updates` uses explicit `--since` filtering, not `Updated` inference.
* **Subjects/Keywords:** `Subject` `subjectScheme`/`valueURI`/`classificationCode` `datacite.rs:1100` vs RAD `Keyword` lower-kebab via `keywords.csv` `ControlledVocabulary::normalize`.
* **Sizes/Formats:** `sizes[]` free `15 pages`/`6 MB`, `formats[]` MIME/extension `application/pdf`.

## Fixture Index (Pinned)

* `tests/fixtures/schema/datacite/4.7/valid-4.7-deltas.json` — JSON API valid for all 4.7 deltas (Poster, Presentation, RAiD/SWHID/Other+info, RelatedItem)
* `tests/fixtures/schema/datacite/4.7/valid-4.7-related-item.xml` — XML kernel valid (same, round-trip via `quick_xml` `datacite.rs: tests`)
* `tests/fixtures/schema/datacite/4.7/invalid-missing-mandatory.json` — missing `titles` (deposit-ready `Err`)
* `tests/fixtures/schema/datacite/4.7/invalid-missing-creators.json` — missing `creators`
* `tests/fixtures/schema/datacite.json` / `tests/fixtures/schema/datacite.xml` — baseline
* `tests/fixtures/schema/datacite-invalid-2-errors.json` — 2-error case
* `tests/fixtures/raid/mint.json` — pinned RAiD mint response used by the import crosswalk; strict missing-required and inference cases are constructed in adjacent unit tests

## Change Log

* 2026-09-03 Slice 0: Added `Raid`/`Swhid`, `Other`, `Poster`/`Presentation`, `RelatedItem`, `relationTypeInformation`, `validate_deposit_ready`, warning pipeline `Omitted` (from `mod.rs:232`), `mapping.rs:194` propagation, `check.rs:1099`, `as_vec` for `doi->identifier`.
* 2026-09-16 Slice 1: Added output contributor/funder profile fields, complete DataCite date vocabulary coverage, `ResearchOutput::validate_deposit_ready`, and warning-aware `ResearchOutput <-> DataCite Record` conversion with focused round-trip coverage.
* 2026-09-17 Slice 2 (RAiD adapter): Added warning-aware `ResearchActivity <-> RAiD Metadata`, strict default export, explicit dated inference, caller-supplied import identity context/name maps, and pinned-fixture coverage.
* 2026-09-17 final slice: Added strict `ResearchActivity -> DataCite Project`, RAD/RAiD structured-export dispatch and inference, warning-aware dry runs, pre-write strict failure, and conflict tests proving canonical `funders`/`relatedResources` cannot be combined with their legacy aliases.
