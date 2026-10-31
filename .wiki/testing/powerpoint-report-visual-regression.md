---
okf_version: "0.1"
type: Plan
title: Automate visual regression tests for report PowerPoint decks
description: Generate representative report decks, render them with LibreOffice on Linux and Windows, and verify both source content and reviewed slide images.
difficulty: medium
impact: medium
package: [acorn-cli]
tags: [testing, report, powerpoint, libreoffice, visual-regression, windows, linux]
status: planned
execution: phased
phase_budget_hours: 5
---

# Goal

Make a named test gate generate actual `acorn report --format powerpoint` output, open each resulting deck through headless LibreOffice, and fail when its slide content or rendered layout regresses. Run the gate on Linux and native Windows with reviewed, platform-specific image baselines. Keep ordinary `make test` usable on hosts without LibreOffice or Poppler.

# Current evidence

- `crates/acorn-cli/src/commands/report/tests/powerpoint.rs` generates decks and checks slide count, category order, interpolated XML, speaker-note source IDs, malformed templates, overflow, and preservation of a prior deck. `crates/acorn-cli/src/commands/report/tests/mod.rs` exercises local deck dispatch, both-format runs, and PDF-only `--combine` behavior. These tests do not open the deck in an office renderer or compare pixels.
- `crates/acorn-cli/src/commands/export/tests/mod.rs` has ignored, explicitly invoked PDF visual tests that use `pdftoppm` and `insta` binary PNG snapshots. Reuse the fixture and artifact discipline, but keep report snapshots in a dedicated report test module and use separate OS baselines.
- The retained `tests/fixtures/report/` project inputs and `tests/fixtures/export/reference.pptx` support deterministic multi-category decks. Create one small, isolated visual fixture so unrelated report-fixture edits do not silently change the accepted images.
- On October 31, 2026, a report deck generated from the existing fixture opened and rendered through LibreOffice on macOS; the PDF and PPTX report tests passed when Chromium and ACORN's cache were writable. This is feasibility evidence, not automated Linux or Windows acceptance.
- [LibreOffice's command-line documentation](https://help.libreoffice.org/latest/en-GB/text/shared/guide/start_parameters.html?DbPAR=BASIC&System=WIN) documents `--headless`, `--convert-to`, `--outdir`, a writable `UserInstallation` profile, and Windows `soffice.com` for console-driven invocations. A unique profile is needed for isolated parallel or repeated test processes.
- The current Windows shell runner readiness plan records that runner 55645 starts a tagged job but cannot publish its final status. This blocks a claim of Windows CI acceptance; it does not block the portable harness or Linux baseline work.

# Test contract and decisions

- The named gate is `make test-report-visual`. It runs ignored report visual tests explicitly. Missing LibreOffice, Poppler, required fonts, or an incompatible pinned version fails that gate with a named prerequisite; the test must not silently skip after invocation. Ordinary `make test` remains independent of these host tools.
- Drive the existing report command with an isolated fixture, explicit window, commit, absolute reference path, and local output directory. Assert the expected `.pptx` exists and contains the expected number and order of category slides. Cover a short slide, a dense slide near accepted capacity, and a slide with a local authored image. The fixture contains no network media.
- Check exact semantic content in OOXML before rendering: project title, category/window label, every selected update summary, expected source IDs in notes, the authored image's media relationship, absence of template placeholders, and no extra category slide. Keep existing failure/preservation tests as the behavioral authority for invalid input. Do not use OCR to decide whether source text is correct.
- Convert the generated PPTX to PDF using headless LibreOffice, then rasterize **every selected slide page** at a fixed resolution with `pdftoppm`. Assert the PDF page count equals the selected category count and compare each page to a reviewed PNG baseline for that OS. Preserve the generated PDF, actual PNGs, and a useful mismatch message as CI artifacts on failure. Do not compare a Linux image to a Windows baseline.
- Use a fresh temporary output directory and unique writable LibreOffice profile per invocation. Allow explicit executable paths for CI and local installations; otherwise resolve Windows `soffice.com` and `pdftoppm.exe`, or Unix `soffice` and `pdftoppm`. Pass paths as process arguments rather than through a shell. Bound the child-process duration and report exit status, stderr, and missing output. A zero exit status without a PDF is a failure.
- Preserve failing output under `target/report-visual/<os>/` outside the test's disposable directory. Linux CI invokes the Make target; the native Windows PowerShell job may invoke the same filtered Cargo test command directly when Make is unavailable.
- Pin the LibreOffice, Poppler, and font set used by each CI baseline. Record versions and image dimensions with the reviewed baseline. Exact PNG comparison is acceptable only inside that pinned environment; a version/font change requires a deliberate baseline review, not an automatic overwrite or a broad tolerance that can hide clipping. Save current output for inspection when a comparison fails.
- LibreOffice rendering proves compatibility with LibreOffice. Microsoft PowerPoint rendering and Office automation are outside this plan.

# Boundaries and dependencies

- No report production behavior, template interpolation, schema, publication, or CLI syntax changes. The visual tests consume existing report output.
- The portable harness and Linux baseline can proceed without the Windows runner. PVR-04 needs a Linux job with the pinned renderer, Poppler tools, and fonts. PVR-05 needs a native Windows host with those tools **and** a runner that completes a tagged job; GSR-02 is the external readiness prerequisite for CI acceptance, not an implementation dependency for the harness.
- Coordinate edits to report test modules with grouped report separators and sponsor highlights. PVR-04 and PVR-05 serialize `.gitlab-ci.yml` and `Makefile` with runner pools, coverage throughput, Podman migration, and other CI/build owners. Visual baselines are reviewed artifacts, not a reason to change production renderers opportunistically.

# Validation

- Run the new gate on a pinned Linux host and a native Windows host. Confirm it fails on a deliberately changed expected slide image and retains the actual PNG and PDF for review, then restore the accepted baseline.
- Verify semantic assertions catch changed update text, missing note IDs, wrong slide order/count, and a missing authored image independently of image comparison.
- Check that `make test` still passes when LibreOffice and Poppler are absent. Run focused report tests, `make lint`, `make test`, and the repository's portability checks after CI wiring is stable.
- Record successful Linux and Windows CI job links, exact tool/font versions, and reviewed baseline images in durable developer documentation before retiring the plan.

# Acceptance criteria

- A named gate generates a report PPTX, opens it with LibreOffice, and verifies semantic content plus the rendered short, dense, and authored-image slides on Linux and Windows.
- Each OS uses its own reviewed baseline under pinned tool and font versions; no test silently skips when the named gate is requested.
- A changed slide image fails with a path to inspect the actual output; changed source text or notes fail semantic assertions even if the pixels happen to look similar.
- The gate is required by dedicated Linux and native Windows CI jobs after their host prerequisites are met; both jobs finish successfully and retain diagnostics on failure.
- Ordinary `make test` and report production behavior remain unchanged, and the developer guide explains how to run and update the baselines.

# Execution phases

## PVR-01 — Generate and open a representative deck

- Status: ready
- Budget: 5 active hours maximum
- Prerequisites: a local host with LibreOffice and Poppler for focused acceptance; existing report/reference fixtures.
- Exact write set: `crates/acorn-cli/src/commands/report/tests/mod.rs`, `crates/acorn-cli/src/commands/report/tests/visual.rs` (new), `tests/fixtures/report/visual/index.json` (new), `tests/fixtures/report/visual/figure.png` (new), `.wiki/testing/powerpoint-report-visual-regression.md`
- Primary integration lock: report command tests
- Outcome: The ignored named test generates a fixed multi-category deck through report dispatch, checks exact slide/notes content, converts the PPTX to PDF with an isolated LibreOffice profile, checks page count, and produces fixed-resolution PNGs with bounded process execution.
- Focused validation: Run the named test directly on the available host; intentionally omit each tool once and verify a clear failure. Run normal focused report tests without the tools.
- Handoff: Prune PVR-01, retain the portable harness and deterministic fixture, and record the successful host/tool versions.

## PVR-02 — Accept a Linux image baseline

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PVR-01; Linux host with agreed LibreOffice, Poppler, and font versions.
- Exact write set: `crates/acorn-cli/src/commands/report/tests/visual.rs`, `tests/fixtures/report/visual/baselines/linux/short.png` (new), `tests/fixtures/report/visual/baselines/linux/dense.png` (new), `tests/fixtures/report/visual/baselines/linux/image.png` (new), `.wiki/testing/powerpoint-report-visual-regression.md`
- Primary integration lock: report visual baselines
- Outcome: Compare all three rendered pages against reviewed Linux images, reject size/content mismatches, and leave actual output available on failure. Keep exact OOXML assertions independent of the PNG comparison.
- Focused validation: Run the named test with the pinned Linux tools; alter one expected image and one source summary separately to prove each failure path, then restore the accepted files.
- Handoff: Prune PVR-02 and record the Linux tool/font fingerprint and baseline review evidence.

## PVR-03 — Accept a native Windows image baseline

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PVR-02; native Windows host with LibreOffice `soffice.com`, Poppler, fonts, and a writable isolated profile. A working GitLab runner is not required for this local phase.
- Exact write set: `crates/acorn-cli/src/commands/report/tests/visual.rs`, `tests/fixtures/report/visual/baselines/windows/short.png` (new), `tests/fixtures/report/visual/baselines/windows/dense.png` (new), `tests/fixtures/report/visual/baselines/windows/image.png` (new), `.wiki/testing/powerpoint-report-visual-regression.md`
- Primary integration lock: report visual baselines
- Outcome: Prove path resolution, profile isolation, conversion, semantic assertions, and reviewed native Windows PNG comparisons using the same fixture and test contract.
- Focused validation: Run the named gate in native Windows PowerShell; rerun after a fresh profile, confirm missing-tool failure, and verify all three PNG comparisons.
- Handoff: Prune PVR-03 and record native OS, tool/font versions, and accepted baseline evidence.

## PVR-04 — Require the visual gate in Linux CI

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PVR-02; a Linux CI execution image or host with the pinned LibreOffice, Poppler, and fonts available before the job starts.
- Exact write set: `Makefile`, `.gitlab-ci.yml`, `.wiki/testing/powerpoint-report-visual-regression.md`
- Primary integration lock: CI/build
- Outcome: Add `make test-report-visual` and a dedicated Linux job that runs it, verifies tool versions, and retains PDF/PNG diagnostics when the test fails. The job fails if its host tools are absent or different from the accepted baseline fingerprint.
- Focused validation: Run the Make target locally and finish one Linux CI job with an inspectable log and artifacts; verify a forced image mismatch makes the job fail.
- Handoff: Prune PVR-04 and record the successful Linux job link and tool versions.

## PVR-05 — Require the visual gate in Windows CI

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PVR-03, PVR-04; GSR-02 runner readiness evidence that a native Windows tagged job can finish and publish its final status; pinned LibreOffice, Poppler, and fonts installed on that runner.
- Exact write set: `.gitlab-ci.yml`, `scripts/report-visual/validate-windows.ps1` (new), `.wiki/testing/powerpoint-report-visual-regression.md`, `.wiki/plan-network.md`, `.wiki/parallel-execution.md`
- Primary integration lock: Windows CI/build
- Outcome: Add a dedicated native Windows PowerShell job that checks prerequisites, runs the same named visual gate, and retains PDF/PNG diagnostics. Keep this job separate from Needle, Podman, and normal report publication.
- Focused validation: Finish a tagged Windows job, inspect its native OS/tool versions and visual assertions, and confirm a forced PNG mismatch fails the job with artifacts.
- Handoff: Prune PVR-05 and retain the successful Windows job link outside the active plan.

## PVR-06 — Document baseline maintenance and close

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PVR-04 and PVR-05 have accepted job links and reviewed images; shared CI/build work is stable.
- Exact write set: `docs/src/learn/developer.md`, `.wiki/testing/powerpoint-report-visual-regression.md` (delete after acceptance), `.wiki/testing/index.md`, `.wiki/index.md`, `.wiki/plan-network.md`, `.wiki/parallel-execution.md`
- Primary integration lock: developer test documentation
- Outcome: Document local invocation, required tools/fonts, OS baseline review, failure artifacts, and CI job evidence; remove the completed plan and every index/network/lane reference.
- Focused validation: Build the book, run `make lint`, `make test`, and portability gates after implementation stabilizes; verify both visual CI jobs and wiki validation before deleting the plan.
- Handoff: Preserve the operational test recipe and accepted job links in developer documentation; retire the plan.
