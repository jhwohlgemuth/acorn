# Report design fixtures

These files contain fictional project and logbook data for PDF layout review. Three entries
marked `global: true` live in the configured project records and appear on the title page.

- `watershed-sensing/index.json`: 22 entries covering field methods, data, infrastructure, research, software, and one global update.
- `materials-discovery/index.json`: 22 entries covering collaboration, engagement, funding, personnel, publication, research, and one global update.
- `grid-resilience/index.json`: 24 entries covering demonstrations, research, software, a longer multipage example, and one global update.

Together the 68 entries cover all 12 logbook categories, all 8 event formats, all 3 states, manual and discovered origins, an archived entry, an unclassified administration entry, and entries with varied amounts of supporting text. Thirty-six summaries have multiple paragraphs; two entries also include long supporting items to exercise bullet wrapping. The reporting window is October 1 through November 1, 2026 (exclusive).
Project metadata also includes fictional PI names, explicit sponsor placeholders for the PDF credit line, and project keywords shown as topic tags in each banner.

March and September 2026 report examples contain 110 substantive, month-specific paragraphs of at least 80 characters after identical recurring project background and mailing-list text are excluded. Those paragraphs average about 331 characters; their medians are 248 and 226 characters by month, and most fall between roughly 100 and 700 characters. This fixture deliberately runs a little longer to test layout tolerance: its 68 summaries average 367 characters, with a 281-character median. Eleven summaries are 80–149 characters, 13 are 150–249, 21 are 250–499, 21 are 500–749, and two are longer (967 and 1,084 characters). The summary validator permits up to 1,200 characters.

Generate the project PDFs locally with:

```sh
target/debug/acorn report --dry-run --config tests/fixtures/report/config.json --root tests/fixtures/report --commit 0000000000000000000000000000000000000000 --from 2026-10-01T00:00:00Z --through 2026-11-01T00:00:00Z --output tmp/pdfs/report/reports --chrome-path "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"
```

No report artifacts are published by this command.
Add `--full-timestamp` to show the year and UTC time in the PDF; the default shows month and day.
The report command reads configured project RAD files and writes separate title, legend, and project PDFs. JSONL intake updates the project RAD files before reporting; it is not a direct multi-project PDF input. The title page and fixed status/category legend are written to `tmp/pdfs/report/reports/title-page.pdf` and `tmp/pdfs/report/reports/legend-page.pdf`; the three project PDFs remain in their versioned project directories.

Use `--combine` to write one local `report.pdf` with the title page, selected project updates, and a final legend page. Combined mode does not require a commit or publish packages and callbacks:

```sh
target/debug/acorn report --combine --config tests/fixtures/report/config.json --root tests/fixtures/report --from 2026-10-01T00:00:00Z --through 2026-11-01T00:00:00Z --output tmp/pdfs/report/combined --chrome-path "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"
```

