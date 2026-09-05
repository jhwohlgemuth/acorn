---
okf_version: "0.1"
---

# Standards Field Matrix — RAD | RAiD 1.x | DataCite 4.7 | DCAT 3 | CodeMeta | Schema.org

**Pin:** DataCite kernel-4.7 `https://schema.datacite.org/meta/kernel-4.7/metadata.xsd` + JSON API, RAiD `https://metadata.raid.org/en/latest/` (ISO 23527:2022), DCAT 3 `https://www.w3.org/TR/vocab-dcat-3/`, CodeMeta `https://codemeta.github.io/`, Schema.org `https://schema.org/`. Reconciled `2026-09-03` against `acorn-schema` at `0.1.76` (Slice 0). Owner: `:.wiki/features/rad-metadata-standards-alignment.md:102`.

Slice 0 fixtures (referenced per row): `tests/fixtures/schema/datacite/4.7/valid-4.7-deltas.json` (Poster/Presentation + RAiD/SWHID/Other+info + RelatedItem), `tests/fixtures/schema/datacite/4.7/valid-4.7-related-item.xml` (Resource + RelatedItems), `tests/fixtures/schema/datacite/4.7/invalid-missing-mandatory.json` (missing `titles`), `tests/fixtures/schema/datacite/4.7/invalid-missing-creators.json`.

## Validity Levels

| Level | Gate | Mandatory Fields | Source |
|---|---|---|---|
| Valid RAD | `Validate` derive | `ResearchActivity.title` len 4..MAX, `ResearchActivityMetadata.identifier` kebabcase, `ContactPoint`, `Sections`, vocab-normalized keywords/technology | `crates/acorn-schema/src/schema/research_activity/mod.rs:160`, `crates/acorn-schema/src/validation/` |
| RAiD-ready | `pid/raid/mod.rs` `Metadata::validate` | `access` (required), `contributor` ≥1 (leader/contact flags), `date.startDate`, `title` Primary≥1, `organization` LeadResearchOrganisation | `crates/acorn-schema/src/schema/pid/raid/mod.rs` |
| Deposit-ready | `validate_deposit_ready(&Record)` / `&Resource` | `doi`/`identifier`, `titles`, `creators`, `publisher`, `publicationYear`, `resourceTypes`/`resourceType` (general) | `crates/acorn-schema/src/schema/standard/datacite.rs:1525` (distinct from `Validate`) |

Recommended `rad-metadata-standards-alignment.md:107` vs optional: `subjects`, `contributors`, `dates`, `relatedIdentifiers`, `descriptions`, `geoLocations`; Optional: `language`, `alternateIdentifiers`, `sizes`, `formats`, `version`, `rightsList`, `fundingReferences`, `relatedItems` (DataCite 4.7 prop 20) — validated but not deposit-blocking.

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
| `sections` (`Sections`, `nested`, `compat`) | `entries[].summary` (local-only) | — | `description[]` (mapped aggregate, not per-entry) | `descriptions[]` `Abstract/Methods/...` (aggregate only) | `descriptions/description` | `dcterms:description` | — | `schema:description` | lossy | aggregate: project export may map `Changelog.summary` as `Description Other`, per-entry as `local-only` warn `rad-metadata-standards-alignment.md:61` | ACORN | 1 | prose | `datacite.xml: descriptions[0]=Abstract` |
| `contact` (`ContactPoint`, `nested`, `compat`) | — | — | `contributor[]` (`leader`/`contact`) | `creators[0]` / `contributors` | `creators/creator` | `dcat:contactPoint` | `author` | `schema:author` | transformed | `ContactPoint` → `Contributor`/`Creator` | ORCID/ROR | 1 | `format_phone_number` etc | `raid` org |
| `notes` (`Option<Other>`) `compat` | — | — | `—` | `—` | `—` | `—` | `—` | `schema:Text` | local-only | — | ACORN | 0..1 | free text | — |
| `changelog` (`Option<Changelog>`) `research-activity-changelog.md` | `entries[]` (`identifier` kebab-unique, `timestamp` RFC3339 offset, `category` Code/Technical/Personnel/Publication/General, `state` Completed/InProgress/Upcoming, `origin` Manual/Discovered + `discovery` provenance, `summary` nonempty≤500, `items` 1..20) | — (refs `ResearchOutput` via future related-resource primitive, not embedded) | `—` (do not map `timestamp` to RAiD interval/metadata timestamps; `category/state/summary/items/origin` local-only) | `—` (do not map `timestamp` to `dateType=Updated`; aggregate `summary→Description Other` opt-in, per-entry `local-only` warn) | `—` | `—` | `—` | ACORN owned `changelog` terms (no false DataCite equiv) | local-only | preserve order, require offset, stable `identifier` | ACORN | 0..1 (`entries` 1..1000) | `timestamp` built-in, `nonempty` | `changelog` whiteboard fixture |

### ResearchActivityMetadata (`research_activity/mod.rs:164`, `rename_all=camelCase`, 27 fields)

| RAD Field | RAiD Path | DataCite JSON | DataCite XML | DCAT 3 | CodeMeta | Schema.org | Classification | Transform | Vocab | Occ | Normalization | Fixture |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| `@context` / `@type` | `—` | `—` | `—` | `—` | `—` | JSON-LD | local-only | — | W3C | 0..1 | CURIE | — |
| `enrichment: Option<Vec<EnrichmentProvenance>>` (`provider`, `source`, `fields`) | `—` | `—` | `—` | `—` | `—` | provenance | local-only | — | ACORN `Provider` (Orcid/Ror/CiteAs/OpenAlex/Osti) | 0..n | `kebabcase` fields | — |
| `classification: Option<ClassificationLevel>` (Unclassified/Confidential/Secret/TopSecret) | `—` | `—` | `—` | `—` | `—` | `schema:DefinedTerm` | local-only | — | ACORN | 0..1 | enum | — |
| `archive: bool` (default false), `draft: bool` (default true) | `—` | `—` | `—` | `—` | `—` | `schema:Boolean` | local-only | — | ACORN | 1 | bool | — |
| `status: Status` (Active/Paused/Completed/Canceled, default Active) | `—` (distinct from dated `contributor`/`organization` roles and `date` interval; also distinct from `ChangelogState`) | `—` | `—` | `—` | `—` | `schema:DefinedTerm` | local-only | — | ACORN | 1 | enum | — |
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
| `keywords: Vec<Keyword>` (default `[]`, lower-kebab-case, `ControlledVocabulary`) | `subject[]` `keyword` `0..n` | `subjects[]` `subject` `datacite.rs:1100` | `subjects/subject` | `dcat:keyword` | `keywords` | `schema:keywords` | exact | `ControlledVocabulary::normalize` | ACORN `keywords.csv` | 1..n (allow empty vec) | `vec_string_trim`, lower-kebab | `datacite.json: Climate change` |
| `technology: Vec<String>` (`vec_string_trim`, `ControlledVocabulary`) | `—` | `—` | `—` | `—` | `programmingLanguage` | `schema:DefinedTerm` | local-only | normalize via `technology.csv` | ACORN | 1..n | `vec_string_trim` | — |
| `sponsors: Option<Vec<String>>` | `organization[]` `Funder` role | `fundingReferences[]` `funderName`/`funderIdentifier` `datacite.rs:733` | `fundingReferences/funderName` | `dcterms:publisher` | `funding` | `schema:sponsor` | transformed | `Vocabulary::Sponsors` via `sponsors.csv`, precedence vs structured `Funding` `output.rs:50` | ROR/Crossref | 0..n | `ControlledVocabulary` | — |
| `partners: Option<Vec<String>>` | `organization[]` `LeadResearchOrganisation` | `contributors[]` `HostInstitution` | `contributors` | `dcterms:contributor` | — | `schema:contributor` | transformed | `partners.csv` | — | 0..n | `ControlledVocabulary` | — |
| `related: Option<Vec<String>>` (related RAD `identifier` k-b) | `relatedRaid[]` / `relatedObject` `category` | `relatedIdentifiers[]` `IsPartOf`/`HasPart` | `relatedIdentifiers` | `dcterms:relation` | — | `schema:isPartOf` | transformed | kebabcase id → `10.26259/`? | ACORN | 0..n | `kebabcase` | — |
| `resources: Option<Vec<Resource>>` (`hardware::Resource` ASIC/CPU/GPU… `output.rs:35`) | `—` (distinct from `ComputePlatform`/`ComputeFacility` `ocp-...`) | `—` (not `resourceType`) | `—` | `—` | — | `schema:DefinedTerm` | local-only | `Resource` variants `Validate` nested (add `#[validate(nested)]` per OCP plan) | ACORN hardware | 0..n | enum | `hardware` |

### ResearchOutput (`research_activity/output.rs:113`, 10 fields, `rename_all=camelCase`)

| RAD Field | DataCite JSON | DataCite XML | RAiD | DCAT 3 | CodeMeta | Classification | Transform | Vocab | Occ | Notes |
|---|---|---|---|---|---|---|---|---|---|---|
| `identifier: Option<String>` (`option_string_trim`) | `relatedIdentifiers[]` / `alternateIdentifiers[]` | `alternateIdentifiers` | `relatedObject` | `dcterms:identifier` | `identifier` | exact | trim | — | 0..1 | provider-native vs global |
| `doi: Option<String>` (`doi`) | `doi` (prop 1) `datacite.rs:515` | `identifier` `DOI` `datacite.rs:1128` | `relatedObject DOI` | `dcat:identifier` | `identifier` | conditionally-required | DOI | DOI | 0..1 | deposit: required via `validate_deposit_ready` |
| `title: String` (`length(min=1)`, `string_trim`) REQUIRED | `titles[0].title` (prop 3) | `titles/title` | `title Primary` | `dcterms:title` | `name` | exact | trim | — | 1 | `datacite.json: Example Dataset Title` |
| `type` (`kind: Option<String>`, `option_string_trim`, `rename="type"`) e.g. `article`/`dataset`/`software`/`poster`/`presentation` | `types.resourceTypeGeneral` `Poster`/`Presentation` `datacite.rs:321` + `resourceType` | `resourceType @resourceTypeGeneral` | `relatedObject type` | `dcat:type_` → IRI `mapping.rs:482` (`Poster`→`https://schema.org/Poster`, `Presentation`→`PresentationDigitalDocument`) | `type` | transformed | DataCite `ResourceTypeGeneral` | 0..1 | `valid-4.7-deltas.json: Poster, Presentation` |
| `publication_date: Option<String>` (`is_partial_date` ISO8601 partial) | `publicationYear` `Option<i32>` `year` + `dates[]` `Issued` `datacite.rs:654` (`timestamp`) | `publicationYear u32` + `dates/date @dateType=Issued` | `date` interval | `dcterms:issued` `year_to_date` `mapping.rs:467` | `datePublished` | transformed | year vs RFC3339 | 0..1 | partial vs complete `timestamp` |
| `contributors: Option<Vec<Contributor>>` (`nested`: `name` len1, `orcid` `orcid`, `corresponding`, `position` first/middle/last, `affiliations: Vec<Affiliation>` `name`+`ror`) | `creators[]` (prop 2) vs `contributors[]` (prop 7, `contributorType`) — distinction not yet modeled; use CRediT where applicable (`ContributorType` `datacite.rs:31` vs `PositionType` vs `Role`) | `creators/creator` vs `contributors/contributor` | `contributor[]` `role` `CreditRole` | `dcat:creator`/`publisher` | `author`/`contributor` | transformed | ORCID/ROR/ISNI | 0..n | `Contributor` flat vs DataCite split creators/contributors (gap `output.rs:113`) | `datacite.json: creators[0] ORCID` |
| `funding: Option<Vec<Funding>>` (`name` len1, `ror` `ror`, `awards: Vec<Award>` `identifier`, `title`, `doi` `doi`, `url` `url`) | `organization Funder` | `fundingReferences[]` `funderName`/`funderIdentifier`/`awardNumber`/`awardURI`/`awardTitle` `datacite.rs:733` | `—` | `dcat:qualifiedRelation` | `funding` | transformed | ROR/Crossref | 0..n | sponsors legacy `Vec<String>` vs structured `Funding` precedence `rad-metadata-standards-alignment.md:180` | `datacite.xml: fundingReferences[0] European Commission` |
| `websites: Option<Vec<Website>>` (`nested`: publisher/repo/OA) | `url` | `—` (XML uses `relatedIdentifier` URL) | `alternateUrl` | `dcat:landingPage`/`distribution` | `url`/`codeRepository` | transformed | URL | 0..n | `Website` `url` |
| `keywords: Vec<Keyword>` | `subjects[]` | `subjects/subject` | `subject` | `dcat:keyword` | `keywords` | exact | `Keyword` normalize | ACORN `keywords.csv` | 0..n | `datacite.json: subjects[0] Climate change` |
| `access: Option<Access>` (`open: bool`, `status` gold/green/closed, `url` `url`, `license` `validate_license`) | `rightsList[]` `rightsURI`/`rightsIdentifier` SPDX | `rightsList/rights` | `access` `AccessType` + `license` | `dcterms:license`/`accessRights` + `mapping.rs` `license`↔`rightsURI` | `license` | transformed | SPDX via `validate_license` | 0..1 | `datacite.xml: rights cc-by-3.0 https://creativecommons.org/licenses/by/3.0/` |

**Gaps in `ResearchOutput` (documented as `unsupported` until Slice 4):** `publisher` distinct from `creators` (publisher `Publisher` `datacite.rs:898` with `publisherIdentifier`), `language` (`is_iso_639_1_language_code`), `alternateIdentifiers`, `relatedIdentifiers` typed (`RAiD`/`SWHID` `datacite.rs:168`), `RelatedItems` (`RelatedItem` `datacite.rs:1128` with `relationTypeInformation`), `version` (`version`), `rights`/`descriptions` typed (`DescriptionType`), `subjects` with `subjectScheme`/`valueURI`, `formats`/`sizes`/`geoLocations` (`GeoLocationPoint`/`Box`/`Polygon`). These are valid DataCite 4.7 but `ResearchOutput` flat strings today `output.rs:113`.

### RAiD Top-Level (`pid/raid/mod.rs` `Metadata`, `deny_unknown_fields`, `Validate`)

| RAiD Field | RAD Source | Classification | Transform | Occ | Vocabulary |
|---|---|---|---|---|---|
| `identifier` (`MetadataIdentifier`) | `meta.identifier` | exact | kebab → `10.26259/` | 0..1 | DOI-like |
| `title[]` (`TitleType` Primary required) | `title` + `subtitle` | transformed | `subtitle` → `AlternativeTitle` | 1..n | — |
| `contributor[]` (`id` ORCID/ISNI, `role` CreditRole, `position`, `leader`, `contact`, `email`) | `contact` + future `contributors` structured | lossy (single `contact` today) | infer `leader`/`contact` false unless `meta` asserts | 1..n | ORCID/ISNI, CreditRole |
| `organization[]` (`id` ROR, `role` LeadResearchOrganisation vs Funder) | `meta.ror` + `sponsors`/`partners`/`contact.affiliation` + `ror` in `ResearchOutput.affiliations` | transformed | `sponsors` → `Funder`, `partners` → `LeadResearchOrganisation` (when single) | 0..n | ROR |
| `date` (`Date` startDate required) | `—` (no activity interval yet; do not infer current date) | lossy | require explicit `startDate`/`endDate` future fields `rad-metadata-standards-alignment.md:130` | 1 | `unix_epoch`/`date` |
| `relatedObject[]` (`id`, `schemaUri` URL, `type` `RelatedObjectIdentifier`, `category` `RelatedObjectCategory`) | `outputs[]` PID-bearing + `doi`/`handle`/`swhid`/`ror` lists | transformed | DOI/Handle/SWHID normalization | 0..n | `RelatedObjectType` |
| `relatedRaid[]` | `related` (kebab ids) | transformed | kebab → RAiD id | 0..n | `RelatedRaidType` |
| `subject[]` (`keyword`) | `meta.keywords` + `outputs.keywords` | exact | `Keyword` lower-kebab | 0..n | ACORN `keywords.csv` |
| `spatialCoverage[]` (`place`) | `—` (future `spatial_coverage`) | local-only | — | 0..n | `Place` |
| `access` / `description` / `alternateIdentifier` / `alternateUrl` / `traditionalKnowledgeLabel` | `—` | local-only | — | 0..n | — |
| `metadata` (`created`/`updated` unix_epoch) | `—` (do not use `Sections` prose or `Changelog.timestamp` as registry timestamps) | local-only | — | 0..1 | — |

Inference rule `rad-metadata-standards-alignment.md:41,168`: **Do not fabricate** `date`, `leader`/`contact` role, `owner`/`registrationAgency`, `CC0` license, or `relatedObject` when RAD absent; gate behind `allow_infer: bool` default false with `ConversionWarning::Approximated` after Slice 5.

### DCAT 3 / Invenio / HuWise / CFF Crosswalk (existing `crosswalk/`)

| Concern | Source Field | Target Field | Mapping | Classification | Vocab |
|---|---|---|---|---|---|
| Identity | `doi` (`String`) `datacite.rs:168` | `identifier` (`StringVec`) `dcat` | `as_vec` `mapping.rs:482` (`Poster` etc) | transformed | DOI ↔ StringVec |
| Publication year | `publication-year` (`Number`) | `issued` (`Date`) | `year_to_date` / `extract_year` `mapping.rs:467` | transformed | year ↔ `2024-01-01` |
| Language | `language` `String` | `language` `StringVec` | `as_vec` / `first_of_vec` `mapping.rs:440` | transformed | ISO 639-1 `is_iso_639_1_language_code` |
| License | `license` `IRI` | `rightsURI` | direct | exact | SPDX `validate_license` |
| Type | `resource-type` `String` | `type_` `IRI` | `resource_type_to_iri` / `iri_to_resource_type` `mapping.rs:482` (`Poster`→`Poster`) | transformed | DataCite ↔ DCAT IRI |

Existing `Standard::crosswalk` `analyzer/check.rs:1044` supports `datacite ↔ dcat ↔ invenio ↔ huwise` via `FieldMapping::apply` `mapping.rs:194` with loss warnings `crosswalk/mod.rs:238` (`Omitted` for missing optional). RAD ↔ standards not yet (Slice 5 after matrix).

## Controlled Vocabularies & Normalization

* **Identifiers:** DOI `doi` `https://doi.org/10.5072/...`, RAiD `10.83962/...` `https://raid.org/10.26259/...`, SWHID `swh:1:cnt:...` `is_attribute_swhid_list`, ROR `https://ror.org/...` `ror`, ORCID `https://orcid.org/...` `orcid`, ISNI `https://isni.org/isni/...` `validate_name_identifier`, Handle `hdl:…` `Handle::is_valid`.
* **Languages:** `is_iso_639_1_language_code` `datacite.rs:516` (e.g. `en`), DCAT `language` as `StringVec` via `as_vec`.
* **Dates:** DataCite `Date.timestamp` RFC3339 offset `Validate(timestamp)`, `publicationYear` `year`, RAiD `unix_epoch`/`date` with `has_start_date`; do not map `Changelog.timestamp` to `Updated` `rad-metadata-standards-alignment.md:175`.
* **Subjects/Keywords:** `Subject` `subjectScheme`/`valueURI`/`classificationCode` `datacite.rs:1100` vs RAD `Keyword` lower-kebab via `keywords.csv` `ControlledVocabulary::normalize`.
* **Sizes/Formats:** `sizes[]` free `15 pages`/`6 MB`, `formats[]` MIME/extension `application/pdf`.

## Fixture Index (Pinned)

* `tests/fixtures/schema/datacite/4.7/valid-4.7-deltas.json` — JSON API valid for all 4.7 deltas (Poster, Presentation, RAiD/SWHID/Other+info, RelatedItem)
* `tests/fixtures/schema/datacite/4.7/valid-4.7-related-item.xml` — XML kernel valid (same, round-trip via `quick_xml` `datacite.rs: tests`)
* `tests/fixtures/schema/datacite/4.7/invalid-missing-mandatory.json` — missing `titles` (deposit-ready `Err`)
* `tests/fixtures/schema/datacite/4.7/invalid-missing-creators.json` — missing `creators`
* `tests/fixtures/schema/datacite.json` / `tests/fixtures/schema/datacite.xml` — baseline
* `tests/fixtures/schema/datacite-invalid-2-errors.json` — 2-error case
* RAiD fixtures to pin next: `tests/fixtures/schema/raid/valid.json` (access+contributor+date+title) + `invalid-missing-required.json`

## Change Log

* 2026-09-03 Slice 0: Added `Raid`/`Swhid`, `Other`, `Poster`/`Presentation`, `RelatedItem`, `relationTypeInformation`, `validate_deposit_ready`, warning pipeline `Omitted` (from `mod.rs:232`), `mapping.rs:194` propagation, `check.rs:1099`, `as_vec` for `doi->identifier`.
* Next: Use this matrix to close Slice 2 primitives (`typed identifier`, `agent identifier`, `dated role`, `related resource`, `funding`, `subject`, `rights`) without speculative types `rad-metadata-standards-alignment.md:112`.
