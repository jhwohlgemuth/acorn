# 🌾 Gather

> Discover persistent identifiers in files, documents, URLs, standard input, and literal text.

## Directory traversal

Directory inputs are searched recursively by default, including files in every descendant directory. When no input, `--text`, or standard input is supplied, `gather` recursively searches the current directory.

Use `--max-depth` to limit traversal. Direct children of an input directory have depth 1, grandchildren have depth 2, and so on. A depth of 0 processes no directory descendants but does not exclude explicitly supplied files.

```sh
# Search the complete directory tree
acorn gather path/to/project

# Search only files directly inside the directory
acorn gather path/to/project --max-depth 1

# Include one nested directory level
acorn gather path/to/project --max-depth 2
```

`--filter` and `--ignore` are applied to the paths found within the selected depth.

## Other inputs

```sh
acorn gather report.docx
acorn gather --text "doi:10.1234/example"
printf 'https://ror.org/01qz5mb56' | acorn gather
```

Non-empty standard input is gathered automatically and can be combined with positional inputs. Remote URLs require network access. Global `--offline` mode rejects remote input and cannot be combined with `--resolve`.

## Output formats

Gather supports `console`, `json`, `markdown`, and `yaml` output. When `--format` is omitted, an interactive terminal receives the console table and piped or file output uses JSON. An explicit format is always honored. Use `--raw` to print only the normalized identifiers, one per line, for piping to another command. Use `-q` or `--quiet` to suppress stdout output.

```sh
acorn gather report.docx --format console
acorn gather report.docx --format yaml --output discoveries.yaml
acorn gather report.docx --raw | sort -u
```

## Research activity candidates

Unless global `--no-local-database` is set, ordinary gather appends every discovered identifier to discovery history and persists project-like discoveries as evolving research activity candidates. DOI, RAiD, ISBN, patent, and ARK values can independently identify a candidate. ORCID, ROR, PIDINST, unknown identifiers, and arbitrary URLs remain in the report and discovery history but do not create or merge candidates by themselves.

See the [persistent identifier lifecycle](../../concepts/persistent-identifiers.md) for how ACORN finds, parses, validates, normalizes, classifies, and persists this evidence.

Candidates merge only when they share an exact normalized project identity. A new identity creates a row with a portable NanoID; one matching row gains missing fields and unique array values; repeated evidence is unchanged. Conflicting populated fields keep their stored value and record the observation as provenance. Evidence matching multiple rows is reported as a conflict without merging those rows.

### Enrichment mapping

Enrichment is additive and evidence-driven. ACORN starts with the identifiers found locally or returned by the selected provider, calls only the resolvers applicable to that entity and URL, and maps explicitly asserted values into candidate fields. Identifiers are normalized before use; arrays are deduplicated; and websites are deduplicated by URL. Keywords and technologies are resolved to canonical values from ACORN's embedded controlled vocabularies, including known aliases; values with no vocabulary match are omitted from the mapped candidate but remain in provider provenance when that source is persisted. At database persistence, a new scalar fills an empty field, while a conflict with an existing populated scalar preserves the stored value and records the new observation as provenance. Unmapped values are not coerced into unrelated fields. Failure of an optional ORCID, GitHub, or GitLab lookup leaves the original provider result intact.

| Source and trigger | Candidate mapping | Guardrails |
|---|---|---|
| CiteAs for a DOI found with `--resolve` | DOI, title, canonical project URL, and categories to `meta.keywords` | Empty title and URL values are ignored. |
| RAiD metadata found with `--resolve` | Primary title, primary or brief description to `notes`, supported alternate identifiers, alternate URLs to `meta.websites`, organization RORs to `meta.ror`, subject keywords to `meta.keywords`, normalized related RAiDs to `meta.related`, and one explicitly flagged contact to `contact` | A primary title is preferred, with the first non-empty title as fallback. Organization names are added only through the role-aware ROR mapping described below. A contributor is never inferred to be the contact; if zero or multiple distinct contributors are flagged as contacts, no singular contact is selected. |
| DOE CODE project search | `osti-project:<code_id>` identity, DOI, title, description to `notes`, repository and related links to `meta.websites`, sponsor names to `meta.sponsors`, and research, contributing, and developing organization names to `meta.partners` | DOE CODE people and organization results remain report-only. |
| `--lab` with a DOE CODE project search | The selected laboratory's canonical ROR to `meta.ror` | The ROR must match the laboratory entry in ACORN's embedded partner or sponsor vocabulary. |
| ORCID on a DOE CODE person result | Preferred display name, ORCID, public email when DOE CODE has none, and the union of public institutions and existing affiliations | Applies only to a valid ORCID on a person result and does not create a research activity candidate. |
| GitHub or GitLab repository on a project candidate | Provider-reported programming language names to `meta.technology` | ACORN checks the canonical URL and additional repository websites. GitHub requires an overt `github.com` repository URL. GitLab requires the configured GitLab domain and credentials. |

RAiD organization classification is driven by the explicit organization role, not by the organization's name. `Funder` maps to `meta.sponsors`; lead or other research organization, partner, contractor, facility, and other organization roles map to `meta.partners`. An organization carrying both funder and non-funder roles is represented in both lists. A single roleless organization is treated as the default lead organization and therefore as a partner; roleless entries in a multi-organization record are not classified. ACORN resolves a display name only when the ROR exists in its embedded sponsor or partner vocabulary. Otherwise, the normalized ROR remains in `meta.ror` without an invented name.

The [persistent identifier lifecycle](../../concepts/persistent-identifiers.md#6-resolve-and-enrich-optional-metadata) describes field precedence, identity matching, CFF intake, and provenance in more detail. JSON and YAML reports include each persisted candidate's `iid` and `action`, plus `created`, `enriched`, `unchanged`, and `conflicts` totals under `summary.candidates`.

```sh
# Discover identifiers without writing discovery history or candidates
acorn --no-local-database gather --text "doi:10.1234/example" --format json

# Resolve supported metadata and create or enrich candidates
acorn gather report.docx --resolve --format yaml
```

Candidates are internal evolving records. Database-backed candidate export and completeness scoring are not currently available.

## DOE CODE searches

Use `--osti` to search DOE CODE for projects or to aggregate credited people and organizations from matching projects.

```sh
acorn gather --osti projects "ACORN"
acorn gather --osti people https://orcid.org/0000-0002-2057-9115
acorn gather --osti projects --organization ORNL --organization-role site-owner
acorn gather --osti organizations ORNL --all --format json
acorn gather --osti projects --lab ORNL --all
```

`--lab` restricts results to projects submitted by one of DOE's 17 national laboratories. Laboratory acronyms are case-insensitive. It sets the organization role to `site-owner` and cannot be combined with `--organization` or `--organization-role`. Each persisted project candidate is enriched with the laboratory's canonical ROR from ACORN's partner and sponsor vocabularies.

DOE CODE searches return one page by default. `--limit` controls the upstream page size, `--offset` selects a later page, and `--all` retrieves every page sequentially. DOI and ORCID resolver URLs are normalized to the bare identifier expected by DOE CODE. `--organization-role` can restrict organization matching to `site-owner`, `research`, `sponsor`, `contributor`, or `developer`; the default matches any credited role.

Project matches create or enrich candidates using the DOE CODE project ID and DOI when available. Gather retains the project title and description, repository and related websites, sponsoring organizations, and research, contributing, and developing partner organizations. When the repository overtly uses `github.com`, gather queries the GitHub API for programming languages. A repository on the configured GitLab domain receives equivalent enrichment when GitLab credentials are available. Language names are recorded in `meta.technology`.

People searches enrich valid ORCID matches with the preferred name, public email, and institution data returned by the ORCID API. If an optional enrichment lookup fails, the original provider data is retained. People and organization searches are report-only. Global `--no-local-database` keeps all DOE CODE results report-only.

OSTI mode treats positional inputs and `--text` values as direct queries rather than files or URLs. It requires network access and is incompatible with file filtering, merge-request gathering, identifier resolution, and schema analysis options.
