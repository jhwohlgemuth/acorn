# Quick Quest: Validate your first RAD file

In about 10 minutes, you will take one research activity data (RAD) file through ACORN's core loop:

<div class="acorn-growth emoji-growth four-stages" aria-label="Quest progress from seed to tree"><span>Download</span><span>Check</span><span>Format</span><span>Export</span></div>

<div style="height: 3rem" aria-hidden="true"></div>

```pikchr
scale = 0.8
Start: circle radius 0.07in fill 0x00662C thickness 0
arrow right 0.45in thick color 0x00662C
Input: box "index.json" wid 1.05in ht 0.52in rad 0.08in fill 0xF3F0FF thickness 0
arrow right 0.45in thick color 0x00662C
Check: box "Check" wid 0.78in ht 0.52in rad 0.08in fill 0xE8F1FB thickness 0
arrow right 0.45in thick color 0x00662C
Format: box "Format" wid 0.82in ht 0.52in rad 0.08in fill 0xE7F3E9 thickness 0
arrow right 0.45in thick color 0x00662C
Export: box wid 0.82in ht 0.52in rad 0.08in fill 0x00662C color 0x00662C
text "Export" color white at Export
arrow right 0.72in thick color 0x00662C
CFF: box "CITATION.cff" wid 1.2in ht 0.52in rad 0.08in fill 0xFFF4D6 thickness 0
arrow right 0.45in thick color 0x00662C
End: circle radius 0.1in fill white thickness 0
circle radius 0.05in fill 0x00662C thickness 0 at End
PDF: box wid 1.2in ht 0.52in rad 0.08in fill 0xFCE8E6 thickness 0 at CFF+(0,-0.72in)
arrow from Export.e right 0.36in then down until even with PDF then to PDF.w thick color 0x00662C
arrow from PDF.e right 0.225in then up until even with End then to End.w thick color 0x00662C
```

<div id="quick-quest-workflow" data-logo-base="../_assets/artifact-logos" data-pikchr-fit hidden></div>

<div style="height: 3rem" aria-hidden="true"></div>

## 📥 Download the project

[Download the quest fixture](../_assets/quick-quest/index.json), or fetch it from the documentation repository with ACORN:

```sh
acorn download https://code.ornl.gov/research-enablement/acorn \
  --filter '^docs/src/_assets/quick-quest/' \
  --output acorn-quick-quest
cd acorn-quick-quest/docs/src/_assets/quick-quest
```

Your tiny project contains one RAD record and its fact-sheet graphic:

```text
quick-quest/
├── index.json
└── lazer-kitten.png
```

![A lazer kitten investigates a green laser in a research lab](../_assets/quick-quest/lazer-kitten.png)

Key fields in the record look like this:

```json
{
  "meta": {
    "id": "quick-quest",
    "media": [{ "contentUrl": "lazer-kitten.png" }]
  },
  "title": "Lazer Kitten Research Lab",
  "subtitle": "A tiny experiment in reusable research storytelling"
}
```

## 🔎 Check it

```sh
acorn --offline check index.json --skip prose
```

You should get a successful exit and no schema findings. ACORN may still offer a readability coaching note. `--offline` makes the lab deterministic; `--skip prose` avoids synchronizing the optional prose analyzer.

## ✨ Preview, then format

```sh
acorn format index.json --dry-run
acorn format index.json
```

The first command shows a diff without writing. The second applies normalized formatting and controlled-vocabulary values. A second dry run should show no remaining changes.

## 📤 Export shareable files

```sh
acorn --offline export index.json --format cff --skip prose
acorn --offline export index.json --format pdf --skip prose
```

Expected artifacts: `CITATION.cff`, beside `index.json`, and `export/quick-quest.pdf`. The PDF uses the downloaded lazer kitten graphic and the full project narrative. Both files come from the same source record, so you can update one file and reproduce every output.

## 🏆 Quest complete

You now have one human-maintainable source plus citation and PDF outputs.

<p class="acorn-next"><strong>Next stop:</strong>&nbsp;Explore the full <a href="../cli-reference/commands/check.md">Check</a>, <a href="../cli-reference/commands/format.md">Format</a>, and <a href="../cli-reference/commands/export.md">Export</a> guides.</p>
