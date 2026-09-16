# Metadata enrichment

ACORN enrichment uses trusted metadata services to fill gaps in research activity data (RAD). It is additive: populated values are preserved, arrays are deduplicated, ambiguous identity matches are left unchanged, and provider failures do not discard successful results from earlier providers.

Enrichment is opt-in because it performs network requests and may add externally asserted metadata. Preview changes before writing when possible:

```sh
acorn format path/to/project/index.json --enrich --dry-run
acorn format path/to/project/index.json --enrich
```

Global `--offline` mode cannot be combined with enrichment. For a configured bucket, download or copy its files locally before running the enrichment workflow; enrichment does not enumerate bucket configuration.

## Providers

Without `--provider`, `acorn format --enrich` runs every provider in this deterministic order: OSTI, OpenAlex, CiteAs, ORCID, then ROR. Use repeated or comma-delimited `--provider` values to select a subset:

```sh
acorn format index.json --enrich --provider openalex,citeas
acorn format index.json --enrich --provider orcid --provider ror
```

Provider names are case-insensitive.

| Provider | When it applies | Data it can add | Environment variables |
|---|---|---|---|
| OSTI | `meta.identifier` is `osti-<code>` or the RAD has a DOI | Missing subtitle and DOI metadata from DOE CODE | None |
| OpenAlex | The RAD has one or more DOIs | Structured research outputs: OpenAlex ID, DOI, title, type, publication date, contributors and affiliations, funding, locations, keywords, and access metadata | `OPENALEX_API_KEY` is optional but recommended. `OPENALEX_API_HOST` optionally overrides `api.openalex.org`. |
| CiteAs | A DOI is not already represented in `meta.outputs` | A basic research output with identifier, DOI, title, type, publication year, and CiteAs URL | `CITEAS_API_TOKEN` is optional. `CITEAS_SERVER_HOST` optionally overrides `citeas.org`. |
| ORCID | The contact has complete names but no identifier, or an identifier with a missing given or family name | An ORCID for one unambiguous exact name match, or missing names from an exact ORCID profile | `ORCID_API_TOKEN` is optional. `ORCID_SERVER_HOST` optionally overrides `pub.orcid.org`. |
| ROR | `meta.ror` is missing and the contact has an affiliation or organization | A ROR for one unambiguous active organization match | `ROR_API_TOKEN` is optional. `ROR_SERVER_HOST` optionally overrides `api.ror.org`. |

The public defaults work without provider environment variables. Tokens are sent only when configured. Host overrides are mainly useful for proxies, compatible deployments, and tests.

In the default order, OpenAlex supplies the richer scholarly-output record and CiteAs acts as a fallback when OpenAlex does not produce an output for a DOI. `gather --enrich` always uses the complete default provider sequence; provider selection is available on `format --enrich`.

```env
# Recommended for OpenAlex usage
OPENALEX_API_KEY=replace-with-your-key

# Optional public-API tokens
CITEAS_API_TOKEN=replace-with-your-token
ORCID_API_TOKEN=replace-with-your-token
ROR_API_TOKEN=replace-with-your-token
```

Do not commit secrets in `.env` files. Environment variables set by the shell take effect through the same provider configuration.

## Matching and conflict safeguards

OpenAlex merges missing metadata into an existing output with the same normalized DOI. CiteAs adds an output only when that DOI is not already represented. Existing non-empty scalar values and non-empty collections win over provider candidates.

ORCID identifier discovery requires an exact normalized given-name and family-name match. When several records match, ACORN uses the contact email or organization to disambiguate; otherwise it reports a conflict and leaves the identifier unchanged. When an identifier is already present, ACORN uses its exact profile to fill only blank given or family names; populated local names are preserved, and the public credit name is not split or inferred. ROR similarly requires an exact active display name or alias and uses the contact email domain when several organizations match.

When a provider changes `title`, `subtitle`, `contact.familyName`, `contact.givenName`, `contact.identifier`, `meta.doi`, `meta.outputs`, or `meta.ror`, ACORN records the provider, lookup source, and changed field paths in `meta.enrichment`. ORCID provenance uses the identifier for profile-based name enrichment and the full name for identifier discovery. Provider failures are retained and the remaining providers continue. Successful changes can therefore be applied even when the command ultimately reports a nonzero status for one or more provider failures.

## How enrichment fits the workflow

The reusable workflow engine follows this shape:

```text
read → check → format → enrich → format → check → link → plan → apply
```

Planning is separated from writing, which lets callers preview a complete change and keeps `--dry-run` non-mutating. The engine is generic over supported metadata standards through ACORN's input/output, validation, serialization, and enrichment traits. The current CLI enrichment mappings target RAD; additional standards can opt in without making the workflow engine schema-specific.

In the command-line workflow:

1. [`gather`](../cli-reference/commands/gather.md) discovers identifiers and candidate evidence from documents, URLs, and text. `--resolve` resolves discovered identifiers, while `--enrich` enriches valid RAD inputs in memory before discovery. Gather never rewrites those source files.
2. [`check`](../cli-reference/commands/check.md) validates the RAD structure and content.
3. [`format --enrich`](../cli-reference/commands/format.md) normalizes the local RAD, runs the selected providers, normalizes the enriched result, previews or applies the combined change, and records enrichment provenance.
4. [`link`](../cli-reference/commands/link.md) and [`export`](../cli-reference/commands/export.md) create linked or presentation artifacts from the maintained RAD.

A review-oriented sequence is:

```sh
acorn gather path/to/project --resolve
acorn check path/to/project/index.json
acorn format path/to/project/index.json --enrich --dry-run
acorn format path/to/project/index.json --enrich
acorn check path/to/project/index.json
```

Use `gather --raw --enrich` when a pipeline needs enriched RAD candidates as JSON Lines without modifying inputs. Use `format --enrich` when the maintained local RAD itself should be updated.
