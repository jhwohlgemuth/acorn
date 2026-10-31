# 📊 Report

`acorn report` selects accepted logbook updates from configured project `index.json` files and creates PDF reports, local PowerPoint decks, or both. Apply reviewed intake entries before reporting; graduation is optional and archived entries remain eligible.

```sh
# Existing PDF publication workflow.
acorn report --config .acorn.json --through 2026-11-01 --commit "$CI_COMMIT_SHA"

# Local PowerPoint decks for an explicit window.
acorn --offline report --config .acorn.json --from 2026-10-01 --through 2026-11-01 \
  --format powerpoint --commit 0123456789abcdef --dry-run

# One combined local PDF and one local deck per selected project.
acorn --offline report --config .acorn.json --from OCT --format pdf,powerpoint \
  --combine --commit 0123456789abcdef --dry-run
```

`--format` defaults to `pdf`. Specify `pdf`, `powerpoint`, or both; repeat `--format` or separate the values with commas. Use `--project ID` to restrict configured projects and `--output PATH` to change the default `.acorn/reports` output directory. `--root PATH` selects the repository checkout containing the configured project paths.

## Reporting window

`--from` is inclusive and `--through` is exclusive. Both accept `YYYY-MM-DD`, `DDMMM`, `MMM`, and RFC 3339 timestamps with an offset. A date names midnight UTC. Month shorthand uses the current UTC year: `--from OCT` begins October 1 and, when `--through` is omitted, ends at November 1; `--through OCT` also means November 1. Supply the following day's date to include an entire final day. `ACORN_REPORT_THROUGH` can supply the upper bound.

An explicit `--from` selects the same window for every requested format. Without it, normal PDF reporting uses each project's previous GitLab manifest `through` as the lower bound, or full history if none exists. When a PDF manifest is replayed for an unchanged upper bound, a deck requested in the same normal run uses that manifest's recorded window. PowerPoint-only and `--combine` runs use full history through the selected upper bound when `--from` is omitted.

Updates without a project stay on the PDF title-page path; they do not become project slides. Projects without selected project updates produce no deck.

## PowerPoint decks and templates

PowerPoint output is one `report.pptx` per selected project, with one slide for each category containing updates in the window. Slides follow schema category order; entries within a category are newest first. The slides include project context, available authored media, and source entry IDs in speaker notes. A category that exceeds the reference slide's capacity fails with an error; an earlier valid deck remains in place.

By default, each project uses `reference.pptx` beside its `index.json`. Use `--reference PATH` to choose another template. A relative path is resolved beside each project's `index.json`; an absolute path is used for every selected project. The reference must contain the named `title`, `subtitle`, and `achievement` slide fields and a `notes` field on its notes slide. `--reference` affects PowerPoint only.

Project artifacts use `<output>/<project-id>/<through>-<commit>/report.pptx`, beside `report.pdf` when both are produced. The directory's upper bound has its punctuation removed, and the commit component uses the first 12 hexadecimal characters of `--commit`. The full selected window appears inside the deck. PowerPoint output needs a commit identity for this path, including with `--combine`; it needs no GitLab pipeline or job ID.

## PDF publication and `--combine`

Normal PDF reporting retains its GitLab PDF package, manifest, and callback workflow. PowerPoint decks stay local and are absent from those PDF packages and callbacks. PowerPoint-only reporting creates local decks without publishing manifests or sending callbacks. `--dry-run` generates local artifacts and PDF manifests, where applicable, without publication or callbacks.

`--combine` applies to PDF only: it writes `<output>/report.pdf` with the selected projects and does not publish per-project PDFs. It never merges PowerPoint decks or changes their count. A PDF-only combined run needs no commit identity; add `--commit` when the selection includes PowerPoint. `--full-timestamp` changes PDF date display only. PDF rendering needs Chrome or Chromium; use `--chrome-path PATH` when browser discovery is insufficient.
