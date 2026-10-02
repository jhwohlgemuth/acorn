# 📤 Export
<blockquote class="blockquote-tag acorn-nutshell">
<p class="blockquote-tag-title acorn-nutshell-title">In a nutshell</p>
<p>Turn one RAD source into a shareable artifact with <code>acorn export &lt;PATH&gt; --format &lt;FORMAT&gt;</code>.</p>
</blockquote>

`acorn export` converts one RAD record or a directory of records into formats such as CFF, PDF, PowerPoint, Markdown, JSON, JSON Lines, YAML, or ZON.

Markdown, `.zonf`, `.jsonl`, and `.ndjson` RAD are accepted as inputs for their supported RAD export targets alongside JSON and YAML. Exported Markdown uses YAML frontmatter with `schema: acorn/research-activity`; its generated `## ASPECT` section includes nested data and model details.

The command creates each selected artifact from the same source record, so a project can maintain its data once and regenerate its public outputs when that data changes.

```pikchr
scale = 0.8
Start: circle radius 0.07in fill 0x00662C thickness 0
arrow right 0.653in thick color 0x00662C
JSON: box wid 1.5in ht 0.52in rad 0.08in fill 0xF3F0FF thickness 0
arrow right 0.653in thick color 0x00662C
Export: box wid 0.8in ht 0.52in rad 0.08in fill 0x00662C color 0x00662C
text "Export" color white at Export
arrow right 0.85in thick color 0x00662C
PDF: box wid 1.5in ht 0.52in rad 0.08in fill 0xFCE8E6 thickness 0
arrow right 0.653in thick color 0x00662C
End: circle radius 0.1in fill white thickness 0
circle radius 0.05in fill 0x00662C thickness 0 at End
PPTX: box wid 1.5in ht 0.52in rad 0.08in fill 0xFBE9E2 thickness 0 at PDF+(0,-0.72in)
CFF: box wid 1.5in ht 0.52in rad 0.08in fill 0xE7F3E9 thickness 0 at PDF+(0,-1.44in)
Project: box wid 1.5in ht 0.52in rad 0.08in fill 0xE8F1FB thickness 0 at PDF+(0,-2.16in)
Bag: box wid 1.5in ht 0.52in rad 0.08in fill 0xFFF4D6 thickness 0 at PDF+(0,-2.88in)
YAML: box wid 1.5in ht 0.52in rad 0.08in fill 0xFFF0F0 thickness 0 at JSON+(0,-0.72in)
Markdown: box wid 1.5in ht 0.52in rad 0.08in fill 0xF2F2F2 thickness 0 at JSON+(0,-1.44in)
arrow from Start.e right 0.3265in then down until even with YAML then to YAML.w thick color 0x00662C
arrow from YAML.e right 0.3265in then up until even with Export then to Export.w thick color 0x00662C
arrow from Start.e right 0.3265in then down until even with Markdown then to Markdown.w thick color 0x00662C
arrow from Markdown.e right 0.3265in then up until even with Export then to Export.w thick color 0x00662C
arrow from Export.e right 0.3265in then down until even with PPTX then to PPTX.w thick color 0x00662C
arrow from PPTX.e right 0.3265in then up until even with End then to End.w thick color 0x00662C
arrow from Export.e right 0.3265in then down until even with CFF then to CFF.w thick color 0x00662C
arrow from CFF.e right 0.3265in then up until even with End then to End.w thick color 0x00662C
arrow from Export.e right 0.3265in then down until even with Project then to Project.w thick color 0x00662C
arrow from Project.e right 0.3265in then up until even with End then to End.w thick color 0x00662C
arrow from Export.e right 0.3265in then down until even with Bag then to Bag.w thick color 0x00662C
arrow from Bag.e right 0.3265in then up until even with End then to End.w thick color 0x00662C
```

<div id="export-workflow-logos" data-logo-base="../../_assets/artifact-logos" data-pikchr-fit hidden></div>

> [!TIP]
> You can see some export command results by visiting the [ORNL Research Activity Index](https://research.ornl.gov), which features a variety of research activity data presented in different formats.

## Example usage
```sh
# Export research activity data to PDF fact sheet
acorn export /path/to/index.json --format pdf

# Create PowerPoint presentations from all research activity data in a directory
acorn export /path/to/project/ --format powerpoint

# Create a CITATION.cff file from a research activity index
acorn export /path/to/project/index.json --format cff

# Convert a JSON array or JSON Lines batch to canonical .jsonl output
acorn export /path/to/project/activities.json --format jsonl

# Print the Application Configuration for Research Enablement JSON Schema
acorn export --schema --standard acre
```

> [!NOTE]
> Most export formats use the `--output` option to specify the output file or directory path. If not provided, ACORN will generate a default output path based on the input path and selected format. For example, `--format pdf` and `--format powerpoint` will generate files in the default export location (`./export/`) with names based on the project parent folder(s), `--format bag` will add `.zip` to the output path (i.e., `--output ./export` will create `./export.zip` and `--output /path/to/bag` will create `/path/to/bag.zip`). CFF output is the exception described below.

BagIt export supports 7z, ZIP, TAR, and TAR.GZ containers. A recognized `--output` suffix selects the format, while an output without an archive suffix defaults to ZIP. Use `--archive-format 7z|zip|tar|tar.gz` as an explicit override:

```sh
acorn export ./research --format bag --output ./deposit.tar.gz
acorn export ./research --format bag --output ./deposit.7z
acorn export ./research --format bag --archive-format tar --output ./deposit
```

An explicit format that conflicts with an output suffix is rejected. `--archive-format` is not accepted for non-BagIt exports.

## JSON Lines batches

Use `--format jsonl` to write newline-delimited JSON. `--format ndjson` is an alias for the same format; output always uses the canonical `.jsonl` suffix.

```sh
# Convert a JSON array to one compact JSON object per line
acorn export activities.json --format jsonl

# Normalize an .ndjson batch to the canonical .jsonl suffix
acorn export activities.ndjson --format jsonl

# Convert a JSON Lines RAD batch to a JSON array
acorn export activities.jsonl --format json
```

Each nonblank physical line must contain exactly one complete JSON record. LF and CRLF line endings are accepted, the final newline is optional on input, and output always ends with one newline. Empty input, blank record lines, malformed JSON, and records that do not match the selected schema fail the entire export with a one-based line number.

Record order and cardinality are preserved when converting among JSON Lines, JSON, YAML, and ZON batch representations. Directory export still writes one output file per source file; it does not combine unrelated sources into one batch. Export discovery recognizes both `.jsonl` and `.ndjson`, while commands such as `check` and `format` keep their existing input sets.

## Standard crosswalks

Use `--to` with JSON, JSON Lines, YAML, or ZON export to convert metadata between supported metadata standards. ACORN can infer the source standard from recognizable fields, but `--from` is recommended for repeatable scripts and for records with an ambiguous shape.

```sh
# Convert one DataCite JSON record to DCAT JSON
acorn export datacite.json --format json --from datacite --to dcat --output ./export

# Convert DCAT JSON to DataCite YAML
acorn export dataset.json --format yaml --from dcat --to datacite --output ./export

# Convert DataCite ZON to DCAT ZON; output uses .zonf
acorn export datacite.zonf --format zon --from datacite --to dcat --output ./export

# Convert an ordered DataCite JSON Lines batch to DCAT JSON Lines
acorn export datacite.ndjson --format jsonl --from datacite --to dcat --output ./export

# Convert a CKAN package_show record or successful Action API response to a DCAT Dataset
acorn export package.json --format json --from ckan --to dcat --output ./export

# Preview a directory conversion without writing files
acorn export ./metadata --format json --from invenio --to dcat --dry-run
```

The source and target may each be `datacite`, `dcat`, `invenio`, or `huwise`. CKAN is supported only as a source for DCAT Dataset output. Some conversions use DataCite as an intermediate representation:

| Source | Direct targets | Targets routed through DataCite |
|---|---|---|
| CKAN | DCAT | None |
| DataCite | DCAT, InvenioRDM, HuWise | None |
| DCAT | DataCite | InvenioRDM, HuWise |
| InvenioRDM | DataCite | DCAT, HuWise |
| HuWise | DataCite | DCAT, InvenioRDM |

Crosswalk export has these constraints:

- The input and output must be JSON, JSON Lines, YAML, or ZON. PDF, PowerPoint, CFF, BagIt, and Markdown are not crosswalk targets.
- A single object, an array of objects, or one object per JSON Lines record is accepted. Directory export writes one target file for each resolved source file.
- CKAN input may be an unwrapped `package_show` result or a successful `{ "success": true, "result": ... }` Action API response. Package-search result envelopes are not supported.
- Source-standard inference checks every JSON Lines record. A batch containing mixed standards fails at the first conflicting record and reports its record index and physical line.
- Conversion can be lossy because the standards have different fields and cardinalities. Normal direct conversions report field-level crosswalk warnings when ACORN has a mapping for that pair.
- CKAN metadata-record timestamps, administrative fields, plugin extensions, malformed resource URLs, and service simplification are reported as warnings. `--strict` fails before writing when any warning is present; `--dry-run` reports the same warnings without writing.
- CKAN conversion emits ACORN's flexible DCAT Dataset representation. It does not claim DCAT-US conformance, create a DCAT Catalog or CatalogRecord, or support DCAT-to-CKAN publishing.
- Warning collection for conversions routed through DataCite remains incomplete; use direct pairs when loss accounting is required.

Run `acorn check --standard <target> <output>` after conversion when the target artifact must pass ACORN's schema and validation checks.

## Citation File Format (`CITATION.cff`)

Use `--format cff` to convert ACORN research activity data (RAD) into a CFF 1.2.0 citation file:

```sh
acorn export /path/to/project/index.json --format cff
```

The generated `/path/to/project/CITATION.cff` includes the RAD title, contact information as author and contact metadata, keywords, and DOI information when available. When narrative sections are present, the research purpose is used as the abstract. ACORN also supplies the CFF version and default citation message.

For CFF export:

- The input must be RAD in JSON, JSONC, YAML, ZON, or canonical/legacy ACORN Markdown. Existing `.cff` input is skipped because it is already in the target format.
- The output is always named `CITATION.cff` and written beside its source RAD file. `--output` does not relocate CFF output.
- ACORN generates at most one `CITATION.cff` per source directory. When several discovered RAD files share a directory, they resolve to the same output path and only one is exported.

Run `acorn check /path/to/project/CITATION.cff` after export to validate the generated citation metadata.

## Chrome selection for PDF export

Use `--chrome-path` to select an installed Chrome or Chromium executable. `CHROME_PATH` provides the same setting through the environment, and an explicit command-line value takes precedence.

```sh
# macOS
acorn --offline export /path/to/index.json --format pdf \
    --chrome-path "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"

# Linux
CHROME_PATH=/usr/bin/google-chrome-stable \
    acorn --offline export /path/to/index.json --format pdf
```

```powershell
# Windows
acorn --offline export C:\path\to\index.json --format pdf `
    --chrome-path "$env:ProgramFiles\Google\Chrome\Application\chrome.exe"
```

When configured, this path is validated and used without checking ACORN's browser cache or downloading Chrome. Without it, offline export uses the cached pinned browser when available and otherwise lets `chromiumoxide` detect an installed browser. Connected export retains the pinned browser download fallback.

Draft PDF and PowerPoint exports do not require authored media. When a RAD has no usable image, ACORN inserts its embedded 1920 × 1280 pixel (3:2) acorn placeholder so the layout remains meaningful. A media path that was explicitly authored but cannot be read is still reported as an error.

## PowerPoint reference template
Use the `--reference` option to provide a PowerPoint template with the styles, layouts, and branding for the exported presentation.

```sh
acorn export /path/to/index.json \
    --format powerpoint \
    --reference /path/to/reference.pptx
```

Place text such as `{{ PLACEHOLDER_NAME }}` in the template where ACORN should insert a RAD value. An example PowerPoint reference template is available in the [ACORN GitLab repository](https://code.ornl.gov/research-enablement/acorn/-/blob/main/tests/fixtures/data/highlight/reference.pptx?ref_type=heads).

### Available placeholders

The following placeholders can be used in your PowerPoint reference template:

#### String values
- `caption` - First image caption
- `challenge` - Challenge description
- `citation` - DOI citation
- `email` - Contact email
- `first` - Contact first name
- `focus` - Research focus area
- `last` - Contact last name
- `managers` - Manager names (joined with `"and"`)
- `mission`
- `notes` - Presentation notes (intended to be added PowerPoint speaker notes)
- `partners` - Partner names (joined with `", "`)
- `programs` - Program names (joined with `"and"`)
- `subtitle`
- `title`

#### Lists (bullet points)
- `achievement`
- `areas` - Research areas
- `impact`
- `technical` - Technical approach

<p class="acorn-next"><strong>Next stop:</strong>&nbsp;Practice the complete loop in the <a href="../../learn/quick-quest.md">Quick Quest</a>.</p>
