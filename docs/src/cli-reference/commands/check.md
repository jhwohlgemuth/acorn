# ✅ Check
<blockquote class="blockquote-tag acorn-nutshell">
<p class="blockquote-tag-title acorn-nutshell-title">In a nutshell</p>
<p>Validate RAD structure, links, and prose with <code>acorn check &lt;PATH&gt;</code>.</p>
</blockquote>

## Example Use
```sh
# Check a specific research activity index
acorn check path/to/project/index.json

# Check Markdown research activity data
acorn check path/to/project/index.md

# Check a Citation File Format file
acorn check path/to/project/CITATION.cff

# Check prose, readability, and links extracted from a PDF
acorn check path/to/report.pdf

# Check all research activity data in a directory
acorn check path/to/project/

# Check a remote document
acorn check https://example.org/project.json

# Check text from standard input
printf 'Text to check' | acorn check

# Check text copied to the system clipboard
acorn check --paste

# Force clipboard content to YAML and validate it as RAD
acorn check --paste --format yaml --standard rads
```

HTTP and HTTPS inputs are downloaded to a temporary file and checked using content-based format detection, with the URL extension as a fallback. This supports extensionless PDF URLs. Remote inputs require network access and are rejected in global `--offline` mode.

ACORN recognizes `.md` and `.markdown` research activity files by their `schema: acorn/research-activity` YAML frontmatter discriminator. Legacy Markdown previously exported by ACORN is also accepted. Ordinary Markdown remains plain text and is not selected by RAD directory discovery.

## Checking text from standard input or the clipboard

When no file, directory, Git selector, filter, or ignore pattern is supplied, `acorn check` reads non-empty piped standard input automatically. Use `--paste` to read text from the system clipboard instead. ACORN infers JSON, JSONC, RAD Markdown, and RAD YAML; other content is checked as plain text.

Use `--format` to override inference with `cff`, `json`, `jsonc`, `markdown`, `text`, or `yaml`. Combine it with `--standard` to select the schema or analyzer independently:

```sh
cat metadata.jsonc | acorn check --format jsonc --standard datacite
acorn check --paste --format cff --standard cff
printf '# Notes\n\nText to check.' | acorn check --format markdown --standard text
```

The aliases `md`, `txt`, and `yml` are accepted. `--format` applies only to piped standard input and clipboard content and conflicts with filesystem, Git, filter, and merge-request selectors.

An explicit filesystem selection or `--paste` takes precedence over piped standard input. Empty implicit standard input preserves the existing behavior of checking the current directory. Clipboard input must contain non-whitespace text, and clipboard or standard-input checks cannot be combined with `--watch`.

## Checking PDF documents

PDF files are discovered in directories and Git changes and are automatically analyzed as document text. ACORN extracts each text-bearing page once, caches the result for the duration of the process, and applies prose, readability, link, and extraction-quality checks. Diagnostics rendered against extracted text include the original one-based PDF page number when the matched text can be located.

Prose diagnostic line and column coordinates refer to the complete source file, not the subset of prose sent to the analyzer. For converted binary documents, coordinates refer to the complete extracted Markdown. When ACORN cannot map a finding to one unambiguous source location, it reports the finding without a line number.

The `document` standard can also be selected explicitly. The previous `docx` name and `pdf` are retained as aliases:

```sh
acorn check report.pdf --standard document
```

PDF extraction is limited to 100 MiB and 1,000 pages. Encrypted, malformed, and wholly image-based PDFs fail with a specific diagnostic. OCR is not bundled; when only some pages require OCR, ACORN checks the extractable pages and reports the skipped page numbers as quality warnings. Filenames are matched case-insensitively, including `.PDF`.

## Watch mode

Use the global `--watch` flag to run immediately and repeat the check whenever the effective input changes:

```sh
acorn --watch check path/to/project/
acorn check --watch --watch-mode poll https://example.org/project.json
```

`--watch-mode auto` uses native filesystem notifications for local inputs and polling for remote inputs, with polling as a fallback when native notifications fail. Use `--watch-mode poll` for network-mounted paths, containers, WSL-mounted paths, or other filesystems where native events are unreliable. Configure polling with `--poll-interval` (`-p`), such as `--poll-interval 5s`. Press `q` or `Ctrl+C` to stop watching.

Before checking, ACORN synchronizes the extensions used by the selected prose analyzer and tries up to three times when synchronization fails. Both one-shot and watch-mode checks use the configured `--poll-interval` between attempts; it defaults to one second. Watch mode does not repeat synchronization on later file changes. If all attempts fail, the check exits; use `--ignore-sync-failure` to let watch mode continue with locally available analyzer extensions instead. Use `--skip prose` to skip prose analyzer synchronization and analysis entirely.

## Checking `CITATION.cff`

ACORN recognizes a directly supplied `.cff` file as [Citation File Format (CFF)](https://citation-file-format.github.io/) data. You do not need to select the standard explicitly:

```sh
acorn check ./CITATION.cff
```

Use `--standard cff` when CFF data is stored with a `.yaml` or `.json` extension:

```sh
acorn check ./citation.yaml --standard cff
```

For CFF input, ACORN checks:

- **Schema**: Parses the YAML or JSON, rejects unknown fields, and validates modeled values such as dates, DOIs, URLs, identifiers, licenses, and nested author or reference data.
- **Prose and readability**: Analyzes the `title`, `abstract`, and `message` text.
- **Links**: Checks repository, license, landing-page, DOI, URL identifier, and reference links.

Link checks require network access. Use `acorn --offline check ./CITATION.cff` or `--disable-website-checks` to skip them.

> [!NOTE]
> Directory and Git-change discovery currently select `.json`, `.jsonc`, and `.yaml` files, but not `.cff` files. Pass each `CITATION.cff` path directly.

## Check categories

The command can validate data against its schema, assess evidence of support for the FAIR principles, analyze prose and readability, check links, find inconsistent or incomplete data, and enforce naming or organization-specific conventions.[^metrics]

FAIR checks evaluate all 41 indicators in the RDA FAIR Data Maturity Model. They report an evidence profile rather than a compliance grade. Native assessment is available for ACORN Research Activity Data, DataCite, DCAT, Huwise, InvenioRDM, and RAiD metadata. See [FAIR evidence assessment](../../concepts/fair-assessment.md) for the evidence states and output contract.

## Customization options
Use command options to select or skip checks and to control when the command exits.

Include `--exit-on-first-error` to stop execution upon encountering the first error.

Bypass verifying the checksum of downloaded artifacts with `--skip-verify-checksum`[^warn]

### Configure Vale

By default, ACORN uses a `vale` executable on `PATH`, reuses an ACORN-managed executable when available, or downloads its pinned Vale release into the platform cache directory. Generated configuration, synchronized styles, and vocabularies are stored alongside the managed installation rather than in the checked project.

Use `--vale-directory <PATH>` to select an existing installation containing `vale` (or `vale.exe` on Windows) and `.vale.ini`. ACORN treats this directory as read-only: it does not download, rewrite configuration, or run `vale sync` there.

In `--offline` mode, ACORN can use a custom, system, or previously cached installation, but it never performs release discovery, downloads, or synchronization. If no installation is available, the command reports how to provide one.

### Skip checks
- `--skip schema` : Skip schema validation checks
- `--skip fair` : Skip FAIR evidence assessment
- `--skip prose` : Skip prose quality checks
- `--skip readability` : Skip readability checks
- `--skip schema,prose` : Skip both schema validation and prose quality checks (this works for any combination of categories)
- `--disable-website-checks` : Disable all website-related checks (link integrity, etc.)

> [!NOTE]
> `--disable-website-checks` is redundant when `acorn --offline` is used for commands that need to be run in offline environments.

### Configure readability
Readability can be configured for desired metric and level by passing options directly to the command line or via a `.env` file. Command line options override `.env` settings.
- `--readability-metric <METRIC>` : Specify which readability metric to use
- Set `READABILITY_METRIC` in your `.env` file to choose the readability metric. Default metric is [`fkgl`](https://acorn.ornl.gov/acorn_lib/analyzer/readability/enum.ReadabilityType.html#variant.FKGL) (Flesch-Kincaid Grade Level).
- Set `MAX_ALLOWED_FKGL` in your `.env` file to define the maximum acceptable FKGL score. Each metric has its own corresponding maximum score variable (e.g., `MAX_ALLOWED_ARI` for Automated Readability Index).

#### Example `.env` file
Configure ACORN to use the Coleman-Liau Index (CLI) readability metric with a maximum allowed score of 14.0 ([default value](https://acorn.ornl.gov/acorn_lib/util/constants/constant.MAX_ALLOWED_CLI.html) is `12.0`):
```env
READABILITY_METRIC=cli
MAX_ALLOWED_CLI=14.0
```

[^metrics]: See the [readability module documentation](https://docs.rs/acorn-lib/latest/acorn_lib/analyzer/readability/enum.ReadabilityType.html) for a full list of available readability metrics.

[^warn]: ⚠️ Skipping checksum verification may expose you to security risks. Use this option with caution.

<p class="acorn-next"><strong>Next stop:</strong>&nbsp;Normalize a checked record with <a href="format.md">Format</a>.</p>
