# 🤖 Format
<blockquote class="blockquote-tag acorn-nutshell">
<p class="blockquote-tag-title acorn-nutshell-title">In a nutshell</p>
<p>Normalize RAD fields and controlled vocabulary values with <code>acorn format &lt;PATH&gt;</code>.</p>
</blockquote>

## Example usage
```sh
# Format a specific research activity index
acorn format path/to/project/index.json

# Format canonical or legacy Markdown RAD
acorn format path/to/project/index.md

# Format canonical ZON RAD in place
acorn format path/to/project/index.zonf

# Format all research activity data in a directory
acorn format path/to/project/

# Perform a dry-run to see proposed changes without modifying files
acorn format path/to/project/index.json --dry-run

# Keep exact aliases but disable fuzzy typo correction
acorn format path/to/project/index.json --fuzzy off

# Preview formatting and provider enrichment together
acorn format path/to/project/index.json --enrich --dry-run

# Format an ACRE configuration file while preserving comments and property order
acorn format .acorn.jsonc --standard acre
```

## ACRE configuration

Use `--standard acre` to format Application Configuration for Research Enablement files. JSON and JSONC use two-space indentation, LF line endings, and a final newline while retaining property order, comments, and existing trailing commas. YAML uses the same indentation and line-ending policy and retains comments. Formatting validates the file first and preserves an omitted legacy `schemaVersion` field.

When the path is a directory, only `.acorn.json`, `.acorn.jsonc`, `.acorn.yaml`, and `.acorn.yml` are selected. An explicit file may use any supported JSON, JSONC, YAML, or YML filename. ACRE formatting is side-effect free and rejects `--config`, `--enrich`, and `--provider`.

## Example output
```diff
  meta:
    keywords:
-   - automatin
+   - automation
    technology:
-   - JavaScript
-   - TypeSpec
    - astro
+   - javascript
    - react
-   - rs
+   - rust
+   - typespec
    sponsors:
-   - DOD
+   - Department of Defense
    ...
  contact:
    jobTitle: Primary Investigator
    givenName: Jasdrey
    familyName: Wohlson
    email: me@example.com
-   telephone: '(123) 456-7890'
+   telephone: '123.456.7890'
    url: https://www.ornl.gov/staff-profile/jason-h-wohlgemuth
-   organization: GSHS
+   organization: Geospatial Science and Human Security Division
+   affiliation: National Security Sciences Directorate
```

## Features

Markdown formatting rewrites the document into ACORN's canonical representation. YAML frontmatter retains lossless structured metadata and ASPECT data, while the body contains the title, prose sections, research fields, contact details, and a generated ASPECT summary. This also upgrades Markdown produced by older ACORN releases. ZON formatting uses ACORN's deterministic strict profile and keeps the `.zonf` extension.
- 🛠️ **Auto-fixing**: Automatically fix common inconsistencies in RAD structure and prose
- 🎨 **Consistent Formatting**: Ensure consistent JSON formatting across all data files
- 🩹 **Resolve Values**: Resolve certain values against controlled vocabularies to ensure correct meaning 
- 🖼️ **Resolve missing images**: Find first image in associated RAD folders and add to metadata if missing

## Fuzzy matching policy

`--fuzzy strict` is the default and applies high-precision vocabulary-specific score and runner-up separation gates. Use `--fuzzy relaxed` to accept more typo and truncation matches, or `--fuzzy off` to retain only exact canonical, declared alias, typed identifier, and normalized matches. Ambiguous, unmatched, and resource-limited values are preserved rather than erased. The selected policy also applies when `--enrich` or configured format hooks run.

## Enrichment

Use `--enrich` to normalize RAD and fill missing metadata through ACORN's ordered provider pipeline. All providers run by default; use repeated or comma-delimited `--provider` values to select OSTI, OpenAlex, CiteAs, ORCID, or ROR. Enrichment requires network access and is rejected in global `--offline` mode before files are written.

```sh
acorn format index.json --enrich
acorn format index.json --enrich --provider openalex,citeas
acorn format index.json --enrich --provider orcid --provider ror --dry-run
```

Existing populated values are preserved, ambiguous identity matches remain unchanged, and successful provider results survive later provider failures. See [Metadata enrichment](../../concepts/enrichment.md) for provider behavior, environment variables, provenance, and the complete workflow.

## Resolved values
- `meta.keywords`: Resolve keywords against [ACORN Keywords Vocabulary](https://code.ornl.gov/research-enablement/acorn/-/blob/main/crates/acorn-schema/assets/constants/keywords.csv)
- `meta.technology`: Resolve technology against [ACORN Technology Vocabulary](https://code.ornl.gov/research-enablement/acorn/-/blob/main/crates/acorn-schema/assets/constants/technology.csv)
- `meta.partners`: Resolve partner names against [ACORN Partners Vocabulary](https://code.ornl.gov/research-enablement/acorn/-/blob/main/crates/acorn-schema/assets/constants/partners.csv)
- `meta.sponsors`: Resolve sponsor names against [ACORN Sponsors Vocabulary](https://code.ornl.gov/research-enablement/acorn/-/blob/main/crates/acorn-schema/assets/constants/sponsors.csv)
- `contact.organization`: Resolve organization name against a given org chart (currently only supports ORNL org chart)
- `contact.affiliation`: Resolve organization name against a given org chart (currently only supports ORNL org chart)

### Examples
- **Keywords**: `"ai"` → *resolves* to `"artificial-intelligence"`
- **Partners**: `"NREL"` → *resolves* to `"National Renewable Energy Laboratory"`
- **Technologies**: `"rs"` → *resolves* to `"rust"`
- **Sponsors**: `"Dept. of Energy"` → *resolves* to `"Department of Energy"`

<p class="acorn-next"><strong>Next stop:</strong>&nbsp;Create a shareable artifact with <a href="export.md">Export</a>.</p>
