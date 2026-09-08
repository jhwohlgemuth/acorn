# SciCrunch RRID contract review

## Plan status

**Not planned.** The reviewed public contract fails the mandatory lookup and search gates below. ACORN will retain portable RRID support and this decision record, but SciCrunch provider implementation is not scheduled. A future proposal requires new contract evidence and a new plan.

## Review snapshot

| Item | Pinned evidence |
|---|---|
| Review date | 2026-09-08 |
| Handbook revision | [`ab802c3853cab90c7b1751a59af9acaf0bd1845b`](https://github.com/SciCrunch/API-Handbook/commit/ab802c3853cab90c7b1751a59af9acaf0bd1845b), authored 2026-02-03 |
| API overview | [RIN overview at the pinned revision](https://github.com/SciCrunch/API-Handbook/blob/ab802c3853cab90c7b1751a59af9acaf0bd1845b/resources-information-network/overview.md) |
| Query examples | [Basic RIN search examples at the pinned revision](https://github.com/SciCrunch/API-Handbook/blob/ab802c3853cab90c7b1751a59af9acaf0bd1845b/resources-information-network/basic-rin-search-examples.md) |
| Terms | [Handbook license at the pinned revision](https://github.com/SciCrunch/API-Handbook/blob/ab802c3853cab90c7b1751a59af9acaf0bd1845b/LICENSE.md) |
| Product description | [RRID system](https://www.rrids.org/rrid-system) |

The handbook describes `POST https://scicrunch.org/api/1/elastic/<index>/_search?api_key=...` as an Elasticsearch pass-through. Its examples are useful discovery material, but they are not a stable typed provider contract.

## Mandatory gate

| Requirement | Published evidence | Decision |
|---|---|---|
| Supported endpoint and revision | The handbook names a production URL and aliases, but provides no API version lifecycle, compatibility policy, deprecation policy, or service commitment. | **FAIL** |
| Credential placement and redaction | The only documented placement is the `api_key` query parameter. URLs can enter traces, proxy logs, errors, and request metadata; no supported header placement is documented. | **FAIL** |
| Stable exact-lookup and search requests | An exact `rrid.curie` query and free-text Elasticsearch examples are shown. The public surface accepts raw Elasticsearch DSL and does not define an allowlisted, versioned request schema. | **FAIL** |
| Stable response and lifecycle fields | Examples expose raw Elasticsearch hits and source fields. No normative field stability, unknown-field policy, canonical-record rule, availability state contract, or identifier-mismatch rule is documented. | **FAIL** |
| Pagination, ranking, totals, and bounds | Examples use `from` and `size`, including `size: 1000`, but do not define a maximum page size, total-result semantics across versions, ranking stability, or safe traversal bounds. | **FAIL** |
| Complete status behavior | The overview lists 200, 400 for an incorrect key, 401 for no key, 404 for an incorrect index or server URI, and 500 for malformed queries. It does not specify absent-identifier, forbidden, rate-limit, timeout, redirect, overload, or partial-response behavior. | **FAIL** |
| Quotas, retry, terms, and retention | No quota, rate-limit headers, `Retry-After`, retry policy, acceptable-use policy, metadata-retention permission, or provenance obligation is defined. The handbook license permits educational, research, and non-profit use but requires permission for commercial use and disclaims support or updates. | **FAIL** |
| Credential configuration and ownership | The RRID site describes developer API-key access, but there is no documented environment-variable convention, header alternative, endpoint-override policy, incident contact, or operational owner for ACORN integration. | **FAIL** |

## Decisions

| Capability | Decision | Consequence |
|---|---|---|
| Exact RRID lookup | **FAIL** | Do not add production DTOs, credentials, HTTP calls, resolver lifecycle integration, or retained SciCrunch payloads. |
| Free-text RRID search | **FAIL** | Do not expose raw Elasticsearch DSL, pagination, registry search, or CLI/TUI search. |

No contract fixtures are retained because the published payloads are examples rather than normative, versioned response fixtures, and their reuse/retention terms do not establish the planned metadata rights. ACORN keeps strict local RRID parsing, discovery, reporting, and authored DataCite relation mapping independent of SciCrunch availability. Users can confirm or search records through the official RRID portal.

## Conditions for review

Reopen exact lookup only when SciCrunch publishes or directly supplies a dated contract covering a stable endpoint and fields, credential-safe authentication, all required lifecycle outcomes, quotas and retry behavior, metadata retention/licensing, and an operational contact. Reopen search separately only when that contract also defines fixed query fields, ranking, pagination totals and maxima, and hard request/result bounds. Pin any approved contract and representative responses before adding production code.
