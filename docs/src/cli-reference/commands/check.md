# ✅ Check
<blockquote class="blockquote-tag acorn-nutshell">
<p class="blockquote-tag-title acorn-nutshell-title">In a Nutshell</p>
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
```

HTTP and HTTPS inputs are downloaded to a temporary file and checked using content-based format detection, with the URL extension as a fallback. This supports extensionless PDF URLs. Remote inputs require network access and are rejected in global `--offline` mode.

ACORN recognizes `.md` and `.markdown` research activity files by their `schema: acorn/research-activity` YAML frontmatter discriminator. Legacy Markdown previously exported by ACORN is also accepted. Ordinary Markdown remains plain text and is not selected by RAD directory discovery.

## Checking PDF documents

PDF files are discovered in directories and Git changes and are automatically analyzed as document text. ACORN extracts each text-bearing page once, caches the result for the duration of the process, and applies prose, readability, link, and extraction-quality checks. Diagnostics rendered against extracted text include the original one-based PDF page number when the matched text can be located.

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

## Check Categories
- 🏛️ **Schema Validation**: Ensure that all data files conform to the expected schema
- ✨ **Prose Quality**: Analyze written content for standards such as grammar, spelling, and word counts
- 👓 **Readability**: Evaluate readability of written content using established metrics[^metrics]
- 🔗 **Link Integrity**: Verify all hyperlinks within the content are valid and reachable
- 📊 **Data Consistency**: Check for dataset consistency and completeness
- 🚦 **Convention Adherence**: Ensure compliance with naming conventions and organization-specific standards

## Customization Options
The `check` command supports several flags and options to customize its behavior (e.g., skipping certain checks, disabling certain behaviors, etc.)

Include `--exit-on-first-error` to stop execution upon encountering the first error.

Bypass verifying the checksum of downloaded artifacts with `--skip-verify-checksum`[^warn]

### Skip Checks
- `--skip schema` : Skip schema validation checks
- `--skip prose` : Skip prose quality checks
- `--skip readability` : Skip readability checks
- `--skip schema,prose` : Skip both schema validation and prose quality checks (this works for any combination of categories)
- `--disable-website-checks` : Disable all website-related checks (link integrity, etc.)

> [!NOTE]
> `--disable-website-checks` is redundant when `acorn --offline` is used for commands that need to be run in offline environments.

### Configure Readability
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
