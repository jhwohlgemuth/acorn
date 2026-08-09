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

Gather supports `console`, `json`, `markdown`, and `yaml` output. When `--format` is omitted, an interactive terminal receives the console table and piped or file output uses JSON. An explicit format is always honored.

```sh
acorn gather report.docx --format console
acorn gather report.docx --format yaml --output discoveries.yaml
```

## Research activity candidates

Unless global `--no-local-database` is set, ordinary gather appends every discovered identifier to discovery history and persists project-like discoveries as evolving research activity candidates. DOI, RAiD, ISBN, patent, and ARK values can independently identify a candidate. ORCID, ROR, PIDINST, unknown identifiers, and arbitrary URLs remain in the report and discovery history but do not create or merge candidates by themselves.

See the [persistent identifier lifecycle](../../concepts/persistent-identifiers.md) for how ACORN finds, parses, validates, normalizes, classifies, and persists this evidence.

Candidates merge only when they share an exact normalized project identity. A new identity creates a row with a portable NanoID; one matching row gains missing fields and unique array values; repeated evidence is unchanged. Conflicting populated fields keep their stored value and record the observation as provenance. Evidence matching multiple rows is reported as a conflict without merging those rows.

With `--resolve`, gather maps only lossless resolver data into a partial candidate, currently supported identifiers, title, and a provider-confirmed project URL. JSON and YAML reports include each persisted candidate's `iid` and `action`, plus `created`, `enriched`, `unchanged`, and `conflicts` totals under `summary.candidates`.

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

`--lab` restricts results to projects submitted by one of DOE's 17 national laboratories. Laboratory acronyms are case-insensitive. It sets the organization role to `site-owner` and cannot be combined with `--organization` or `--organization-role`.

DOE CODE searches return one page by default. `--limit` controls the upstream page size, `--offset` selects a later page, and `--all` retrieves every page sequentially. DOI and ORCID resolver URLs are normalized to the bare identifier expected by DOE CODE. `--organization-role` can restrict organization matching to `site-owner`, `research`, `sponsor`, `contributor`, or `developer`; the default matches any credited role.

Project matches create or enrich candidates using the DOE CODE project ID and DOI when available. The project title and provider-confirmed project URL are retained as candidate data. People and organization searches are report-only. Global `--no-local-database` keeps all DOE CODE results report-only.

OSTI mode treats positional inputs and `--text` values as direct queries rather than files or URLs. It requires network access and is incompatible with file filtering, merge-request gathering, identifier resolution, and schema analysis options.
