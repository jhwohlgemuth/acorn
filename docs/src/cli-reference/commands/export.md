# 📤 Export
> Export research activity data to various formats for analysis and sharing

This command allows you to export research activity data from ACORN into formats such as CFF, PDF, Markdown, YAML, or PPTX. With `export`, you can easily share your research with sponsors, collaborators, or the general public in a variety of contexts.

`export` supports exporting individual research activity indices or entire directories containing multiple indices.

ACORN allows you to maintain research activity data as persistent, interconnected [single sources of truth](https://en.wikipedia.org/wiki/Single_source_of_truth). As such, you can easily create a variety of output artifacts while ensuring consistency and accuracy across all selected formats.

```mermaid
graph LR
    data("JSON</br>(index.json)") --> export{Export}
    export --> PDF("PDF</br>(fact sheet)")
    export --> PPTX("PPTX</br>(presentation)")
    export --> CFF("CFF</br>(CITATION.cff)")
    export --> Convert("JSON | YAML | Markdown</br>(add to project)")
    export --> BAG("BagIt</br>(archive)")
    export --> more("More to come...</br>(Markdown, YAML, etc.)")
```

> [!TIP]
> You can see some export command results by visiting the [ORNL Research Activity Index](https://research.ornl.gov), which features a variety of research activity data presented in different formats.

## Example Usage
```sh
# Export research activity data to PDF fact sheet
acorn export /path/to/index.json --format pdf

# Create PowerPoint presentations from all research activity data in a directory
acorn export /path/to/project/ --format powerpoint

# Create a CITATION.cff file from a research activity index
acorn export /path/to/project/index.json --format cff
```

> [!NOTE]
> Most export formats use the `--output` option to specify the output file or directory path. If not provided, ACORN will generate a default output path based on the input path and selected format. For example, `--format pdf` and `--format powerpoint` will generate files in the default export location (`./export/`) with names based on the project parent folder(s), `--format bag` will add `.zip` to the output path (i.e., `--output ./export` will create `./export.zip` and `--output /path/to/bag` will create `/path/to/bag.zip`). CFF output is the exception described below.

## Citation File Format (`CITATION.cff`)

Use `--format cff` to convert ACORN research activity data (RAD) into a CFF 1.2.0 citation file:

```sh
acorn export /path/to/project/index.json --format cff
```

The generated `/path/to/project/CITATION.cff` includes the RAD title, research purpose as the abstract, contact information as author and contact metadata, keywords, and DOI information when available. ACORN also supplies the CFF version and default citation message.

For CFF export:

- The input must be RAD in JSON, JSONC, or YAML. Existing `.cff` input is skipped because it is already in the target format.
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

## PowerPoint Reference Template
Customize the PowerPoint export by providing a reference template using the `--reference` option. This allows you to define specific styles, layouts, and branding for your presentations.

```sh
acorn export /path/to/index.json \
    --format powerpoint \
    --reference /path/to/reference.pptx
```

The reference template allows you to specify which values are used and where, using placeholder text in the format `{{ PLACEHOLDER_NAME }}`. During export, ACORN will replace these placeholders with corresponding data from the associated research activity data. You can find an example PowerPoint reference template in the [ACORN GitLab repository](https://code.ornl.gov/research-enablement/acorn/-/blob/main/tests/fixtures/data/highlight/reference.pptx?ref_type=heads).

### Available Placeholders

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
