# Human acceptance test plan

This handbook tests ACORN as a person would use it. It is written for a tester who is comfortable copying commands into PowerShell but does not need to know Rust, JSON schemas, databases, containers, or metadata standards.

The companion Gherkin behavior specification is stored at `tests/acceptance/features/acorn_cli.feature`. Scenario tags and IDs match this handbook. The Gherkin is human-readable rather than automatically executable; this page remains the copy-paste execution layer until Cucumber step definitions are added.

| Gherkin tag | Meaning |
| --- | --- |
| `@baseline` | Run during ordinary acceptance testing |
| `@network` | Requires internet access and a third-party service |
| `@interactive` or `@manual` | Requires a person to observe or stop the command |
| `@owner` | Requires adult/repository-owner supervision and authorization |
| `@gap` | Confirms a currently incomplete interface fails clearly |
| `@optional` | Depends on a build feature or installed program |

The baseline tests work on disposable copies under `target/qa-acceptance/`. Optional tests are clearly labeled when they require the internet, Chrome, Git, a container runtime, credentials, or an account-changing action.

## What ACORN is for

ACORN manages research activity data (RAD) and the artifacts and workflows around it.

| Capability | Likely use case | Commands |
| --- | --- | --- |
| Validate content | Find malformed metadata, broken links, difficult prose, and incomplete research records | `check` |
| Normalize content | Fix common values, formatting, contact data, vocabularies, and missing media references | `format` |
| Add linked data | Produce JSON-LD with standards-based semantic context | `link` |
| Publish artifacts | Create PDF, PowerPoint, Markdown, YAML, JSON, CFF, and BagIt output | `export` |
| Convert standards | Crosswalk DataCite, DCAT, InvenioRDM, and HuWise metadata | `export --from ... --to ...` |
| Discover identifiers | Find DOI, ORCID, ROR, arXiv, ISBN, RAiD, Handle, and other identifiers in text and documents | `gather` |
| Resolve and catalog research | Look up identifier metadata and retain discovery/candidate history | `gather --resolve` |
| Move research content | Download remote buckets or import local/remote content and BagIt archives | `download`, `import` |
| Manage local models | Inspect GGUF metadata, download model weights, and configure inference applications | `import model`, `download model`, `sync` |
| Describe APIs | Turn an OpenAPI document into an ACORN endpoint resource | `import spec` |
| Identify software | Calculate and verify Software Hash Identifiers for files, directories, and Git objects | `swhid` |
| Diagnose a machine | Report system, memory, network, GPU, and software readiness | `doctor` |
| Support agents | Extract ACORN's embedded agent skill | `skill` |
| Run interactive/services | Open the TUI or run/create GitLab bots and runners | `tui`, `serve`, `create` |

The CLI also supports JSON, JSONC, YAML, Markdown RAD, CFF, DataCite, DCAT, InvenioRDM, HuWise, RAiD, plain text, PDF, and Word document inputs where appropriate. Global controls cover offline operation, local SQLite or DuckDB storage, watch mode, concurrency, cache lifetime, verbosity, and quiet output.

## Rules for the tester

1. Open one PowerShell window at the repository root. Keep using that same window so the setup variables remain available.
2. Run the setup once. Each later code block is one test step.
3. A green `[PASS]` is a pass. A red error, thrown `[FAIL]`, missing artifact, crash, hang, or result that contradicts the stated expectation is a fail.
4. Warnings about writing quality are valid test results unless the step says there should be no warnings.
5. Do not run an **owner-supervised** test without the repository owner's help. Those tests can change accounts, create containers, download large files, or expose a service.
6. Record the ACORN version, Windows version, test ID, result, and the complete terminal output for every failure.

Suggested result record:

```text
Test ID:
Result: PASS / FAIL / SKIP
ACORN version:
Computer/OS:
What happened:
Expected:
Screenshot or pasted output:
```

## 0. Setup

### SETUP-01 — Build the exact checked-out source

Skip this step only when testing an installed release. A successful build ends with `Finished`.

```powershell
cargo build --package acorn-cli --bin acorn
if ($LASTEXITCODE -ne 0) { throw "[FAIL] ACORN did not build" }
Write-Host "[PASS] ACORN built successfully" -ForegroundColor Green
```

### SETUP-02 — Create a disposable test workspace

This copies fixtures rather than editing the originals. It also creates environment variables used below and isolates normal output beneath `target/qa-acceptance`.

For a source build:

```powershell
. .\scripts\qa\Setup-AcornAcceptance.ps1 -AcornPath .\target\debug\acorn.exe
```

For an installed release, use this instead:

```powershell
. .\scripts\qa\Setup-AcornAcceptance.ps1 -AcornPath (Get-Command acorn).Source
```

Expected: the script prints paths for the binary, inputs, outputs, database, and optional browser. Do not close this PowerShell window.

## 1. Installation and command discovery

### CLI-01 — Verify the executable reports a version

```powershell
$text = & $env:ACORN_BIN --version 2>&1
$code = $LASTEXITCODE
$text
Confirm-ExitCode -Actual $code -Expected 0 -Test "Version command"
if ($text -notmatch '^acorn-cli \d+\.\d+\.\d+') { throw "[FAIL] Version is not in the expected form" }
```

Expected: output similar to `acorn-cli 0.1.76`.

### CLI-02 — Verify all supported top-level commands are advertised

```powershell
$help = & $env:ACORN_BIN --help 2>&1 | Out-String
$code = $LASTEXITCODE
$required = 'check','create','doctor','download','export','format','gather','import','link','swhid','skill','serve','sync','tui'
$missing = $required | Where-Object { $help -notmatch "(?m)^  $_\s" }
$help
Confirm-ExitCode -Actual $code -Expected 0 -Test "Top-level help"
if ($missing) { throw "[FAIL] Missing commands: $($missing -join ', ')" }
Write-Host "[PASS] All command families are advertised" -ForegroundColor Green
```

### CLI-03 — Verify every command can explain itself

```powershell
$commands = @(
    @('check'), @('create'), @('create','runner'), @('create','bot'),
    @('doctor'), @('download'), @('download','model'), @('export'),
    @('format'), @('gather'), @('import'), @('import','model'),
    @('import','spec'), @('link'), @('swhid'), @('skill'),
    @('serve'), @('serve','bot'), @('sync'), @('tui')
)
foreach ($command in $commands) {
    & $env:ACORN_BIN @command --help *> $null
    if ($LASTEXITCODE -ne 0) { throw "[FAIL] Help failed for: acorn $($command -join ' ')" }
}
Write-Host "[PASS] Every supported command returned help" -ForegroundColor Green
```

### CLI-04 — Verify aliases work

```powershell
& $env:ACORN_BIN --offline --no-local-database lint "$env:ACORN_QA_INPUT\valid-rad\index.json" --disable-website-checks --skip prose *> $null
$lintCode = $LASTEXITCODE
& $env:ACORN_BIN --no-local-database fmt "$env:ACORN_QA_INPUT\format-rad\index.json" --dry-run --no-color *> $null
$formatCode = $LASTEXITCODE
& $env:ACORN_BIN --offline --no-local-database harvest --text "doi:10.1038/nature12373" --raw *> $null
$gatherCode = $LASTEXITCODE
if (@($lintCode, $formatCode, $gatherCode) | Where-Object { $_ -ne 0 }) { throw "[FAIL] One or more aliases failed" }
Write-Host "[PASS] lint, fmt, and harvest aliases work" -ForegroundColor Green
```

## 2. Check research data and documents

These tests use `--skip prose` so the baseline does not need to download or synchronize the external prose analyzer. Website checks are disabled unless a test explicitly exercises the internet.

### CHECK-01 — Check a valid RAD JSON file

```powershell
& $env:ACORN_BIN --offline --no-local-database check "$env:ACORN_QA_INPUT\valid-rad\index.json" --disable-website-checks --skip prose --all
Confirm-ExitCode -Actual $LASTEXITCODE -Expected 0 -Test "Valid RAD JSON check"
```

Expected: a results table appears and the schema count is zero. A readability warning is acceptable.

### CHECK-02 — Detect several errors in malformed RAD

```powershell
$report = "$env:ACORN_QA_OUTPUT\malformed-check.txt"
& $env:ACORN_BIN --offline --no-local-database check "$env:ACORN_QA_INPUT\malformed-rad.json" --disable-website-checks --skip prose --no-fail *>&1 | Tee-Object -FilePath $report
$code = $LASTEXITCODE
Confirm-ExitCode -Actual $code -Expected 0 -Test "Malformed RAD is reported with --no-fail"
Confirm-Text -Path $report -Pattern "Deserialization error" -Test "Malformed fields were detected"
```

Expected: ACORN reports more than one deserialization or schema problem rather than stopping after the first one.

### CHECK-03 — Check supported RAD encodings and metadata standards

```powershell
$cases = @(
    @('valid-rad\index.jsonc','rads'),
    @('rad.yaml','rads'),
    @('rad.md','rads'),
    @('CITATION.cff','cff'),
    @('raid.json','raid'),
    @('datacite.json','datacite'),
    @('dcat.json','dcat'),
    @('invenio-single.json','invenio'),
    @('huwise.json','huwise')
)
foreach ($case in $cases) {
    $path = if ($case[0] -match '^(datacite|dcat|invenio|huwise)') { "$env:ACORN_QA_INPUT\schema\$($case[0])" } else { "$env:ACORN_QA_INPUT\$($case[0])" }
    & $env:ACORN_BIN --offline --no-local-database check $path --standard $case[1] --disable-website-checks --skip prose --no-fail *> $null
    if ($LASTEXITCODE -ne 0) { throw "[FAIL] $($case[1]) check failed" }
}
Write-Host "[PASS] JSONC, YAML, Markdown, CFF, RAiD, DataCite, DCAT, InvenioRDM, and HuWise inputs were accepted" -ForegroundColor Green
```

### CHECK-04 — Check text from standard input

```powershell
"Text supplied through standard input. It has several words and another sentence." |
    & $env:ACORN_BIN --offline --no-local-database check --all --skip prose,link,crosswalk,schema,quality
Confirm-ExitCode -Actual $LASTEXITCODE -Expected 0 -Test "Standard-input text check"
```

Expected: the report identifies the temporary source as `stdin.txt`.

### CHECK-05 — Check a PDF as a document

```powershell
& $env:ACORN_BIN --offline --no-local-database check "$env:ACORN_QA_INPUT\gather\gather.pdf" --standard document --disable-website-checks --skip prose --no-fail
Confirm-ExitCode -Actual $LASTEXITCODE -Expected 0 -Test "PDF document check"
```

Expected: ACORN extracts text and reports document checks. Font-encoding warnings from PDF extraction are acceptable; a crash is not.

### CHECK-06 — Verify terse, skip, and alternate-readability options

```powershell
foreach ($metric in 'fkgl','ari','cli') {
    & $env:ACORN_BIN --offline --no-local-database check "$env:ACORN_QA_INPUT\valid-rad\index.json" --disable-website-checks --skip prose,link --readability-test $metric --terse *> $null
    if ($LASTEXITCODE -ne 0) { throw "[FAIL] Readability metric failed: $metric" }
}
Write-Host "[PASS] Terse output, skipped checks, and three readability metrics work" -ForegroundColor Green
```

### CHECK-07 — Reject a remote source in offline mode

```powershell
& $env:ACORN_BIN --offline --no-local-database check "https://example.org/project.json" --skip prose
$code = $LASTEXITCODE
Confirm-NonzeroExitCode -Actual $code -Test "Offline mode rejects remote check input"
```

Expected: the error says the remote source cannot be used offline.

### CHECK-08 — Check clipboard text and restore the clipboard

```powershell
$previousClipboard = Get-Clipboard -Raw -ErrorAction SilentlyContinue
try {
    Set-Clipboard -Value 'doi:10.1038/nature12373 is mentioned in this clipboard test.'
    & $env:ACORN_BIN --offline --no-local-database check --paste --format text --standard text --skip prose,link,crosswalk,schema,quality
    Confirm-ExitCode -Actual $LASTEXITCODE -Expected 0 -Test "Clipboard text check"
} finally {
    Set-Clipboard -Value $previousClipboard
}
```

## 3. Format and link RAD

### FORMAT-01 — Preview formatting without changing the file

```powershell
$file = "$env:ACORN_QA_INPUT\format-rad\index.json"
$before = (Get-FileHash -LiteralPath $file -Algorithm SHA256).Hash
& $env:ACORN_BIN --no-local-database format $file --dry-run --no-color
$code = $LASTEXITCODE
$after = (Get-FileHash -LiteralPath $file -Algorithm SHA256).Hash
Confirm-ExitCode -Actual $code -Expected 0 -Test "Format dry run"
if ($before -ne $after) { throw "[FAIL] Dry run changed the file" }
Write-Host "[PASS] Dry run left the source unchanged" -ForegroundColor Green
```

Expected: a diff shows vocabulary normalization such as `rs` to `rust`, but the checksum stays unchanged.

### FORMAT-02 — Apply formatting

```powershell
$file = "$env:ACORN_QA_INPUT\format-rad\index.json"
& $env:ACORN_BIN --no-local-database format $file
$code = $LASTEXITCODE
Confirm-ExitCode -Actual $code -Expected 0 -Test "Format writes normalized RAD"
Confirm-Text -Path $file -Pattern '"rust"' -Test "Technology was normalized"
Confirm-Text -Path $file -Pattern 'Department of Energy' -Test "Sponsor was normalized"
```

### FORMAT-03 — Verify offline policy is explicit

```powershell
& $env:ACORN_BIN --offline --no-local-database format "$env:ACORN_QA_INPUT\valid-rad\index.json" --dry-run
$code = $LASTEXITCODE
Confirm-NonzeroExitCode -Actual $code -Test "Unsupported offline format mode is rejected clearly"
```

Expected in the current CLI: a clear message says offline mode is not implemented for `format`.

### LINK-01 — Preview linked-data output without changing files

```powershell
$file = "$env:ACORN_QA_INPUT\link-rad\index.json"
$before = (Get-FileHash -LiteralPath $file -Algorithm SHA256).Hash
& $env:ACORN_BIN --no-local-database link $file --dry-run
$code = $LASTEXITCODE
$after = (Get-FileHash -LiteralPath $file -Algorithm SHA256).Hash
Confirm-ExitCode -Actual $code -Expected 0 -Test "Link dry run"
if ($before -ne $after) { throw "[FAIL] Link dry run changed the source" }
Write-Host "[PASS] Link preview left the source unchanged" -ForegroundColor Green
```

### LINK-02 — Create JSON-LD

```powershell
$file = "$env:ACORN_QA_INPUT\link-rad\index.json"
& $env:ACORN_BIN --no-local-database link $file
$code = $LASTEXITCODE
$jsonld = "$env:ACORN_QA_INPUT\link-rad\index.jsonld"
Confirm-ExitCode -Actual $code -Expected 0 -Test "JSON-LD creation"
Confirm-Path -Path $jsonld -Test "JSON-LD file exists"
Confirm-Text -Path $jsonld -Pattern 'schema.org' -Test "JSON-LD contains linked-data context"
```

## 4. Gather identifiers

### GATHER-01 — Find and normalize identifiers in a text file

```powershell
$report = "$env:ACORN_QA_OUTPUT\identifiers.raw.txt"
& $env:ACORN_BIN --offline --no-local-database gather "$env:ACORN_QA_INPUT\identifiers.txt" --raw | Tee-Object -FilePath $report
$code = $LASTEXITCODE
Confirm-ExitCode -Actual $code -Expected 0 -Test "Identifier discovery"
Confirm-Text -Path $report -Pattern '10.1038/nature12373' -Test "DOI was found"
Confirm-Text -Path $report -Pattern '0000-0002-1825-0097' -Test "ORCID was found"
Confirm-Text -Path $report -Pattern '2101.00001' -Test "arXiv identifier was found"
```

### GATHER-02 — Produce JSON, YAML, and Markdown reports

```powershell
foreach ($format in 'json','yaml','markdown') {
    $destination = "$env:ACORN_QA_OUTPUT\gather.$format"
    & $env:ACORN_BIN --offline --no-local-database gather "$env:ACORN_QA_INPUT\identifiers.txt" --format $format --output $destination
    if ($LASTEXITCODE -ne 0) { throw "[FAIL] Gather $format output failed" }
    if (-not (Test-Path -LiteralPath $destination)) { throw "[FAIL] Gather did not create $destination" }
}
Write-Host "[PASS] Gather wrote JSON, YAML, and Markdown reports" -ForegroundColor Green
```

### GATHER-03 — Extract identifiers from a PDF

```powershell
& $env:ACORN_BIN --offline --no-local-database gather "$env:ACORN_QA_INPUT\gather\gather.pdf" --raw > "$env:ACORN_QA_OUTPUT\pdf-identifiers.txt"
$code = $LASTEXITCODE
Confirm-ExitCode -Actual $code -Expected 0 -Test "PDF identifier gathering"
Confirm-Text -Path "$env:ACORN_QA_OUTPUT\pdf-identifiers.txt" -Pattern '10.1145/73560.73576' -Test "A DOI was extracted from the PDF"
```

### GATHER-04 — Respect recursive depth

```powershell
$tree = "$env:ACORN_QA_INPUT\depth-test"
New-Item -ItemType Directory -Force -Path "$tree\child\grandchild" | Out-Null
'doi:10.1000/direct' | Set-Content "$tree\direct.txt"
'doi:10.1000/child' | Set-Content "$tree\child\child.txt"
'doi:10.1000/grandchild' | Set-Content "$tree\child\grandchild\grandchild.txt"
& $env:ACORN_BIN --offline --no-local-database gather $tree --max-depth 1 --raw > "$env:ACORN_QA_OUTPUT\depth-one.txt"
$code = $LASTEXITCODE
Confirm-ExitCode -Actual $code -Expected 0 -Test "Gather depth limit"
Confirm-Text -Path "$env:ACORN_QA_OUTPUT\depth-one.txt" -Pattern '10.1000/direct' -Test "Direct child was included"
if (Select-String -LiteralPath "$env:ACORN_QA_OUTPUT\depth-one.txt" -Pattern 'grandchild' -Quiet) { throw "[FAIL] Grandchild exceeded max depth" }
Write-Host "[PASS] Deeper descendants were excluded" -ForegroundColor Green
```

### GATHER-05 — Persist discoveries in an isolated SQLite database

```powershell
& $env:ACORN_BIN --offline --database-backend sqlite --database-path "$env:ACORN_QA_DATABASE" gather --text "doi:10.1038/nature12373" --format json > "$env:ACORN_QA_OUTPUT\database-gather.json"
$code = $LASTEXITCODE
Confirm-ExitCode -Actual $code -Expected 0 -Test "SQLite-backed gather"
Confirm-Path -Path $env:ACORN_QA_DATABASE -Test "SQLite database was created"
```

## 5. Export and publish

### EXPORT-01 — Export RAD to YAML in a chosen output directory

```powershell
$destination = "$env:ACORN_QA_OUTPUT\yaml"
New-Item -ItemType Directory -Force -Path $destination | Out-Null
& $env:ACORN_BIN --offline --no-local-database export "$env:ACORN_QA_INPUT\valid-rad\index.json" --format yaml --output $destination
$code = $LASTEXITCODE
Confirm-ExitCode -Actual $code -Expected 0 -Test "YAML export command"
Confirm-Path -Path "$destination\index.yaml" -Test "YAML was written to the requested directory"
```

Expected: `output/yaml/index.yaml` exists. If ACORN writes beside the source instead, record a failure because `--output` was not honored.

### EXPORT-02 — Export RAD to Markdown

```powershell
$destination = "$env:ACORN_QA_OUTPUT\markdown"
New-Item -ItemType Directory -Force -Path $destination | Out-Null
& $env:ACORN_BIN --offline --no-local-database export "$env:ACORN_QA_INPUT\valid-rad\index.json" --format markdown --output $destination
$code = $LASTEXITCODE
Confirm-ExitCode -Actual $code -Expected 0 -Test "Markdown export command"
Confirm-Path -Path "$destination\index.md" -Test "Markdown was written to the requested directory"
Confirm-Text -Path "$destination\index.md" -Pattern 'Valid Project' -Test "Markdown contains the project title"
```

### EXPORT-03 — Create and validate `CITATION.cff`

```powershell
$source = "$env:ACORN_QA_INPUT\valid-rad\index.json"
& $env:ACORN_BIN --offline --no-local-database export $source --format cff
$exportCode = $LASTEXITCODE
$cff = "$env:ACORN_QA_INPUT\valid-rad\CITATION.cff"
Confirm-ExitCode -Actual $exportCode -Expected 0 -Test "CFF export"
Confirm-Path -Path $cff -Test "CITATION.cff exists beside the source"
& $env:ACORN_BIN --offline --no-local-database check $cff --disable-website-checks --skip prose --no-fail
Confirm-ExitCode -Actual $LASTEXITCODE -Expected 0 -Test "Generated CFF can be checked"
```

### EXPORT-04 — Export JSON Schemas for supported standards

```powershell
$standards = 'rads','cff','datacite','dcat','invenio','huwise','raid'
foreach ($standard in $standards) {
    $schema = "$env:ACORN_QA_OUTPUT\$standard.schema.json"
    & $env:ACORN_BIN --offline --no-local-database export --schema --standard $standard --format json --raw > $schema
    if ($LASTEXITCODE -ne 0) { throw "[FAIL] Schema export failed for $standard" }
    if ((Get-Item -LiteralPath $schema).Length -lt 100) { throw "[FAIL] Schema was unexpectedly small for $standard" }
    Get-Content -LiteralPath $schema -Raw | ConvertFrom-Json | Out-Null
}
Write-Host "[PASS] Seven supported standards produced valid JSON Schemas" -ForegroundColor Green
```

Expected: RAD, CFF, DataCite, DCAT, InvenioRDM, HuWise, and RAiD schemas are non-empty JSON. Dublin Core can be selected for checks but currently has no generated schema.

### EXPORT-05 — Crosswalk DataCite to DCAT

```powershell
$destination = "$env:ACORN_QA_OUTPUT\crosswalk"
New-Item -ItemType Directory -Force -Path $destination | Out-Null
& $env:ACORN_BIN --offline --no-local-database export "$env:ACORN_QA_INPUT\schema\datacite.json" --format json --from datacite --to dcat --output $destination
$code = $LASTEXITCODE
$result = "$destination\datacite.json"
Confirm-ExitCode -Actual $code -Expected 0 -Test "DataCite-to-DCAT crosswalk"
Confirm-Path -Path $result -Test "Crosswalk output exists"
Confirm-Text -Path $result -Pattern 'Example Dataset Title' -Test "Crosswalk retained the title"
```

### EXPORT-05B — Exercise every supported crosswalk pairing

```powershell
$sources = @{
    datacite = 'datacite.json'
    dcat = 'dcat.json'
    invenio = 'invenio-single.json'
    huwise = 'huwise.json'
}
$failures = @()
foreach ($from in $sources.Keys) {
    foreach ($to in $sources.Keys | Where-Object { $_ -ne $from }) {
        $destination = "$env:ACORN_QA_OUTPUT\crosswalk-$from-to-$to"
        New-Item -ItemType Directory -Force -Path $destination | Out-Null
        & $env:ACORN_BIN --offline --no-local-database export "$env:ACORN_QA_INPUT\schema\$($sources[$from])" --format json --from $from --to $to --output $destination *> $null
        if ($LASTEXITCODE -ne 0) { $failures += "$from -> $to" }
    }
}
if ($failures) { throw "[FAIL] Crosswalk pairings failed: $($failures -join ', ')" }
Write-Host "[PASS] All 12 crosswalk pairings completed" -ForegroundColor Green
```

### EXPORT-06 — Create a PDF fact sheet

Skip if setup printed `Browser: not detected`.

```powershell
if (-not $env:ACORN_QA_CHROME) {
    Write-Host "[SKIP] PDF export: Chrome or Edge was not detected" -ForegroundColor Yellow
} else {
    $destination = "$env:ACORN_QA_OUTPUT\pdf"
    & $env:ACORN_BIN --offline --no-local-database export "$env:ACORN_QA_INPUT\highlight\index.json" --format pdf --chrome-path $env:ACORN_QA_CHROME --output $destination
    $code = $LASTEXITCODE
    Confirm-ExitCode -Actual $code -Expected 0 -Test "PDF export"
    $pdf = Get-ChildItem -LiteralPath $destination -Filter *.pdf | Select-Object -First 1
    if (-not $pdf -or $pdf.Length -lt 10000) { throw "[FAIL] PDF is missing or unexpectedly small" }
    Write-Host "[PASS] PDF exists and is non-empty: $($pdf.FullName)" -ForegroundColor Green
}
```

Manual visual check: open the PDF and confirm text is readable, images are not stretched, no content is cut off, and any ASPECT chart has legible labels.

### EXPORT-07 — Create a PowerPoint from a reference template

```powershell
$destination = "$env:ACORN_QA_OUTPUT\powerpoint"
& $env:ACORN_BIN --offline --no-local-database export "$env:ACORN_QA_INPUT\highlight\index.json" --format powerpoint --reference "$env:ACORN_QA_INPUT\highlight\reference.pptx" --show-aspect-labels --show-aspect-scores --output $destination
$code = $LASTEXITCODE
Confirm-ExitCode -Actual $code -Expected 0 -Test "PowerPoint export"
$pptx = Get-ChildItem -LiteralPath $destination -Filter *.pptx | Select-Object -First 1
if (-not $pptx -or $pptx.Length -lt 10000) { throw "[FAIL] PowerPoint is missing or unexpectedly small" }
Write-Host "[PASS] PowerPoint exists and is non-empty: $($pptx.FullName)" -ForegroundColor Green
```

Manual visual check: open the presentation and confirm placeholders were replaced, the image appears, speaker notes are sensible, and PowerPoint does not request a repair.

### EXPORT-08 — Create verified BagIt archives

```powershell
$archives = @(
    @('zip','deposit.zip'),
    @('7z','deposit.7z'),
    @('tar','deposit.tar'),
    @('tar.gz','deposit.tar.gz')
)
$failures = @()
foreach ($archiveCase in $archives) {
    $archive = "$env:ACORN_QA_OUTPUT\$($archiveCase[1])"
    & $env:ACORN_BIN --offline --no-local-database export "$env:ACORN_QA_INPUT\bag-source" --format bag --archive-format $archiveCase[0] --output $archive *> $null
    if ($LASTEXITCODE -ne 0 -or -not (Test-Path -LiteralPath $archive)) { $failures += $archiveCase[0] }
}
if ($failures) { throw "[FAIL] BagIt archive formats failed: $($failures -join ', ')" }
Write-Host "[PASS] ZIP, 7z, TAR, and TAR.GZ BagIt exports passed verification" -ForegroundColor Green
```

Expected: the archive is created and ACORN's built-in post-export verification passes. A ZIP file left behind after an error is still a failure.

### EXPORT-09 — Preview export without writing

```powershell
$destination = "$env:ACORN_QA_OUTPUT\dry-export"
& $env:ACORN_BIN --offline --no-local-database export "$env:ACORN_QA_INPUT\valid-rad\index.json" --format pdf --output $destination --dry-run
$code = $LASTEXITCODE
Confirm-ExitCode -Actual $code -Expected 0 -Test "Export dry run"
if (Test-Path -LiteralPath $destination) { throw "[FAIL] Export dry run created output" }
Write-Host "[PASS] Export dry run wrote no artifact" -ForegroundColor Green
```

### EXPORT-10 — Combine multiple RAD records into one JSON artifact

```powershell
$source = "$env:ACORN_QA_INPUT\combine"
$destination = "$env:ACORN_QA_OUTPUT\combined"
New-Item -ItemType Directory -Force -Path $source, $destination | Out-Null
Copy-Item "$env:ACORN_QA_INPUT\valid-rad\index.json" "$source\one.json"
Copy-Item "$env:ACORN_QA_INPUT\highlight\index.json" "$source\two.json"
& $env:ACORN_BIN --offline --no-local-database export $source --format json --combine --output $destination
$code = $LASTEXITCODE
Confirm-ExitCode -Actual $code -Expected 0 -Test "Combined JSON export"
$combined = Get-ChildItem -LiteralPath $destination -Filter *.json | Select-Object -First 1
if (-not $combined) { throw "[FAIL] Combined export did not create one JSON artifact" }
$records = Get-Content -LiteralPath $combined.FullName -Raw | ConvertFrom-Json
if (@($records).Count -lt 2) { throw "[FAIL] Combined artifact has fewer than two records" }
Write-Host "[PASS] Combined artifact contains multiple records" -ForegroundColor Green
```

## 6. Import research content and API specifications

### IMPORT-01 — Import a local research-content directory

```powershell
$destination = "$env:ACORN_QA_OUTPUT\imported"
& $env:ACORN_BIN --offline --no-local-database import "$env:ACORN_QA_INPUT\valid-rad" --output $destination
$code = $LASTEXITCODE
Confirm-ExitCode -Actual $code -Expected 0 -Test "Local content import"
Confirm-Path -Path "$destination\index.json" -Test "Imported RAD exists"
```

### IMPORT-02 — Filter and flatten an import

```powershell
$destination = "$env:ACORN_QA_OUTPUT\flattened"
& $env:ACORN_BIN --offline --no-local-database import "$env:ACORN_QA_INPUT\valid-rad" --filter 'index\.json$' --flatten --output $destination
$code = $LASTEXITCODE
Confirm-ExitCode -Actual $code -Expected 0 -Test "Filtered, flattened import"
Confirm-Path -Path "$destination\index.json" -Test "Selected file was flattened"
if (Test-Path -LiteralPath "$destination\00.png") { throw "[FAIL] Filter allowed an unwanted image" }
Write-Host "[PASS] Filter excluded unrelated files" -ForegroundColor Green
```

### IMPORT-03 — Refuse to overwrite unless `--clobber` is present

```powershell
$destination = "$env:ACORN_QA_OUTPUT\clobber-test"
& $env:ACORN_BIN --offline --no-local-database import "$env:ACORN_QA_INPUT\valid-rad" --output $destination *> $null
if ($LASTEXITCODE -ne 0) { throw "[FAIL] Initial import failed" }
& $env:ACORN_BIN --offline --no-local-database import "$env:ACORN_QA_INPUT\valid-rad" --output $destination *> $null
$withoutClobber = $LASTEXITCODE
Confirm-NonzeroExitCode -Actual $withoutClobber -Test "Import protects existing content"
& $env:ACORN_BIN --offline --no-local-database import "$env:ACORN_QA_INPUT\valid-rad" --output $destination --clobber *> $null
Confirm-ExitCode -Actual $LASTEXITCODE -Expected 0 -Test "Import clobber replaces selected content"
```

### IMPORT-04 — Generate an endpoint resource from OpenAPI

```powershell
$preview = "$env:ACORN_QA_OUTPUT\endpoint-preview.json"
& $env:ACORN_BIN --offline --no-local-database import spec "$env:ACORN_QA_INPUT\openapi.yaml" --name qa::api --domain api.example.org --dry-run > $preview
$code = $LASTEXITCODE
Confirm-ExitCode -Actual $code -Expected 0 -Test "OpenAPI dry-run import"
Confirm-Text -Path $preview -Pattern 'list-projects' -Test "Endpoint operation was imported"
$written = "$env:ACORN_QA_OUTPUT\endpoint.json"
& $env:ACORN_BIN --offline --no-local-database import spec "$env:ACORN_QA_INPUT\openapi.yaml" --name qa::api --domain api.example.org --output $written
Confirm-ExitCode -Actual $LASTEXITCODE -Expected 0 -Test "OpenAPI file import"
Confirm-Path -Path $written -Test "Endpoint JSON was written"
```

### IMPORT-05 — Re-import the BagIt archive

Run this only after EXPORT-08 passes.

```powershell
$archive = "$env:ACORN_QA_OUTPUT\deposit.zip"
if (-not (Test-Path -LiteralPath $archive)) {
    Write-Host "[SKIP] BagIt import: EXPORT-08 did not produce an archive" -ForegroundColor Yellow
} else {
    $destination = "$env:ACORN_QA_OUTPUT\bag-import"
    & $env:ACORN_BIN --offline --no-local-database import $archive --output $destination
    $code = $LASTEXITCODE
    Confirm-ExitCode -Actual $code -Expected 0 -Test "BagIt archive import and checksum verification"
    Confirm-Path -Path "$destination\index.json" -Test "BagIt payload was imported"
}
```

### IMPORT-06 — Read JSON, JSONC, and YAML configuration files

```powershell
$configs = @(
    "$env:ACORN_REPO\tests\fixtures\config\with_local_buckets.json",
    "$env:ACORN_REPO\tests\fixtures\config\with_buckets.jsonc",
    "$env:ACORN_QA_CONFIG\local.yaml"
)
for ($index = 0; $index -lt $configs.Count; $index++) {
    $destination = "$env:ACORN_QA_OUTPUT\configured-$index"
    & $env:ACORN_BIN --offline --no-local-database import --config $configs[$index] --output $destination *> $null
    if ($LASTEXITCODE -ne 0) { throw "[FAIL] Configuration import failed: $($configs[$index])" }
    if (-not (Test-Path -LiteralPath $destination)) { throw "[FAIL] Configuration produced no output" }
}
Write-Host "[PASS] JSON, JSONC, and YAML configurations were accepted" -ForegroundColor Green
```

## 7. Software identifiers

### SWHID-01 — Hash the same file deterministically

```powershell
$file = "$env:ACORN_QA_INPUT\identifiers.txt"
$first = (& $env:ACORN_BIN --no-local-database swhid $file --raw | Out-String).Trim()
$firstCode = $LASTEXITCODE
$second = (& $env:ACORN_BIN --no-local-database swhid $file --raw | Out-String).Trim()
$secondCode = $LASTEXITCODE
if ($firstCode -ne 0 -or $secondCode -ne 0) { throw "[FAIL] File SWHID calculation failed" }
if ($first -ne $second -or $first -notmatch '^swh:1:cnt:[0-9a-f]{40}$') { throw "[FAIL] File SWHID was not deterministic or valid" }
Write-Host "[PASS] Deterministic file SWHID: $first" -ForegroundColor Green
```

### SWHID-02 — Verify a calculated identifier

```powershell
$file = "$env:ACORN_QA_INPUT\identifiers.txt"
$id = (& $env:ACORN_BIN --no-local-database swhid $file --raw | Out-String).Trim()
& $env:ACORN_BIN --no-local-database swhid $file --verify $id
Confirm-ExitCode -Actual $LASTEXITCODE -Expected 0 -Test "SWHID verification"
```

### SWHID-03 — Hash directory and Git objects

```powershell
& $env:ACORN_BIN --no-local-database swhid "$env:ACORN_QA_INPUT\valid-rad" --kind directory --raw
if ($LASTEXITCODE -ne 0) { throw "[FAIL] Directory SWHID failed" }
$repository = "$env:ACORN_QA_ROOT\swhid-git"
New-Item -ItemType Directory -Force -Path $repository | Out-Null
Copy-Item "$env:ACORN_QA_INPUT\valid-rad\index.json" "$repository\index.json"
Push-Location $repository
try {
    git init --quiet
    git config user.email 'acorn-qa@example.org'
    git config user.name 'ACORN QA'
    git add index.json
    git commit --quiet -m 'Add RAD fixture'
    git tag -a v1.0.0 -m 'Acceptance test release'
    $gitCases = @(
        @('git-blob','HEAD:index.json'),
        @('git-tree','HEAD'),
        @('revision','HEAD'),
        @('release','v1.0.0')
    )
    foreach ($case in $gitCases) {
        & $env:ACORN_BIN --no-local-database swhid . --kind $case[0] --reference $case[1] --raw
        if ($LASTEXITCODE -ne 0) { throw "[FAIL] Git SWHID failed for $($case[0])" }
    }
    & $env:ACORN_BIN --no-local-database swhid . --kind snapshot --raw
    if ($LASTEXITCODE -ne 0) { throw "[FAIL] Git snapshot SWHID failed" }
} finally {
    Pop-Location
}
Write-Host "[PASS] Directory, blob, tree, revision, release, and snapshot SWHIDs were calculated" -ForegroundColor Green
```

## 8. Diagnostics, model configuration, and agent support

### DOCTOR-01 — Run all diagnostics

```powershell
& $env:ACORN_BIN --no-local-database doctor
Confirm-ExitCode -Actual $LASTEXITCODE -Expected 0 -Test "Doctor diagnostics"
```

Expected: system, memory, network, GPU, and software findings are shown. Missing optional software is a diagnostic result, not necessarily a test failure.

### DOCTOR-02 — Produce a machine-readable report

```powershell
$report = "$env:ACORN_QA_OUTPUT\doctor.json"
& $env:ACORN_BIN --no-local-database doctor --report > $report
$code = $LASTEXITCODE
Confirm-ExitCode -Actual $code -Expected 0 -Test "Doctor JSON report"
Get-Content -LiteralPath $report -Raw | ConvertFrom-Json | Out-Null
Write-Host "[PASS] Doctor report is valid JSON" -ForegroundColor Green
```

### SYNC-01 — Preview VS Code model configuration

```powershell
$target = "$env:ACORN_QA_OUTPUT\chatLanguageModels.json"
& $env:ACORN_BIN --no-local-database sync --model-file "$env:ACORN_QA_INPUT\models.txt" --assume-models --vscode --force --vscode-config $target --dry-run --no-color
$code = $LASTEXITCODE
Confirm-ExitCode -Actual $code -Expected 0 -Test "VS Code sync preview"
if (Test-Path -LiteralPath $target) { throw "[FAIL] Sync dry run wrote the target file" }
Write-Host "[PASS] Sync preview made no changes" -ForegroundColor Green
```

### SYNC-02 — Write isolated app configurations

```powershell
$models = "$env:ACORN_QA_INPUT\models.txt"
$targets = @(
    @('--vscode','--vscode-config',"$env:ACORN_QA_OUTPUT\vscode.json"),
    @('--opencode','--opencode-config',"$env:ACORN_QA_OUTPUT\opencode.jsonc"),
    @('--llama-swap','--llama-swap-config',"$env:ACORN_QA_OUTPUT\llama-swap.yaml"),
    @('--goose','--goose-config',"$env:ACORN_QA_OUTPUT\goose.yaml")
)
foreach ($target in $targets) {
    & $env:ACORN_BIN --no-local-database sync --model-file $models --assume-models --force @target *> $null
    if ($LASTEXITCODE -ne 0) { throw "[FAIL] Sync failed for $($target[0])" }
    if (-not (Test-Path -LiteralPath $target[2])) { throw "[FAIL] Sync did not write $($target[2])" }
}
Write-Host "[PASS] Four isolated application configurations were written" -ForegroundColor Green
```

### SKILL-01 — Extract ACORN's agent instructions

This writes one cached skill file and may update the clipboard.

```powershell
$skillOutput = & $env:ACORN_BIN --no-local-database skill 2>&1 | Out-String
$code = $LASTEXITCODE
$skillOutput
Confirm-ExitCode -Actual $code -Expected 0 -Test "Agent skill extraction"
if ($skillOutput -notmatch 'SKILL\.md') { throw "[FAIL] Skill output did not identify SKILL.md" }
```

## 9. Interactive and watch behavior

These are manual because they intentionally keep running until the tester stops them.

### WATCH-01 — Re-run a check when a file changes

Run the first block, wait for the initial report, then open another PowerShell window and run the second block. Return to the first window and press `q` or `Ctrl+C` after the second report.

First window:

```powershell
& $env:ACORN_BIN --offline --no-local-database --watch --watch-mode poll --poll-interval 1s check "$env:ACORN_QA_INPUT\valid-rad\index.json" --disable-website-checks --skip prose --no-fail
```

Second window:

```powershell
(Get-Item "$env:ACORN_QA_INPUT\valid-rad\index.json").LastWriteTime = Get-Date
```

Expected: exactly one initial run and at least one new run after the timestamp changes; stopping returns control to the prompt.

### TUI-01 — Open and exit the terminal interface

```powershell
& $env:ACORN_BIN --offline --database-path "$env:ACORN_QA_DATABASE" tui
```

Expected: a readable interactive screen opens, navigation keys respond, and the displayed quit key exits cleanly. Resize the terminal once and confirm the interface redraws without corruption.

### GIT-01 — Select files changed by a Git commit

This creates a tiny disposable Git repository; it does not touch the ACORN repository's history.

```powershell
$gitTest = "$env:ACORN_QA_ROOT\git-selector"
New-Item -ItemType Directory -Force -Path $gitTest | Out-Null
Copy-Item "$env:ACORN_QA_INPUT\valid-rad\index.json" "$gitTest\index.json"
Push-Location $gitTest
try {
    git init --quiet
    git config user.email 'acorn-qa@example.org'
    git config user.name 'ACORN QA'
    git add index.json
    git commit --quiet -m 'Add RAD fixture'
    (Get-Content index.json -Raw).Replace('Valid Project','Valid Project Updated') | Set-Content index.json
    git add index.json
    git commit --quiet -m 'Update RAD fixture'
    & $env:ACORN_BIN --offline --no-local-database check --commit HEAD --disable-website-checks --skip prose --no-fail
    Confirm-ExitCode -Actual $LASTEXITCODE -Expected 0 -Test "Git commit selector"
} finally {
    Pop-Location
}
```

Expected: the report includes `index.json`. A release that changes source selection should repeat this pattern for `format --dry-run`, `link --dry-run`, and `export --dry-run`.

## 10. Optional connected tests

Run these only on a normal internet connection. They can be slower or depend on third-party uptime.

### NET-01 — Check live links and synchronize the prose analyzer

```powershell
& $env:ACORN_BIN --no-local-database check "$env:ACORN_QA_INPUT\valid-rad\index.json" --all --no-fail
Confirm-ExitCode -Actual $LASTEXITCODE -Expected 0 -Test "Connected prose and link checks"
```

Expected: ACORN runs schema, prose, readability, quality, and website checks. A deliberately invalid fixture URL may be reported as a content problem.

### NET-02 — Resolve a DOI citation

```powershell
& $env:ACORN_BIN --no-local-database gather --text "doi:10.1038/nature12373" --resolve --raw --citation-format apa > "$env:ACORN_QA_OUTPUT\citation.txt"
$code = $LASTEXITCODE
Confirm-ExitCode -Actual $code -Expected 0 -Test "DOI resolution"
Confirm-Text -Path "$env:ACORN_QA_OUTPUT\citation.txt" -Pattern '2013' -Test "Resolved citation contains publication details"
```

### NET-03 — Search DOE CODE

```powershell
& $env:ACORN_BIN --no-local-database gather --osti projects "ACORN" --lab ORNL --limit 5 --format json > "$env:ACORN_QA_OUTPUT\osti.json"
$code = $LASTEXITCODE
Confirm-ExitCode -Actual $code -Expected 0 -Test "DOE CODE project search"
Get-Content -LiteralPath "$env:ACORN_QA_OUTPUT\osti.json" -Raw | ConvertFrom-Json | Out-Null
Write-Host "[PASS] DOE CODE returned valid JSON" -ForegroundColor Green
```

### NET-04 — Preview model metadata without downloading weights

```powershell
& $env:ACORN_BIN --no-local-database import model Qwen/Qwen2.5-0.5B-Instruct-GGUF --no-fallback --dry-run
Confirm-ExitCode -Actual $LASTEXITCODE -Expected 0 -Test "Hugging Face model metadata preview"
```

Expected: GGUF filenames, quantizations, revisions, and sizes are reported; no model weight is downloaded.

### NET-05 — Preview model download selection

```powershell
& $env:ACORN_BIN --no-local-database download model Qwen/Qwen2.5-0.5B-Instruct-GGUF --quantization Q4_K_M --local-dir "$env:ACORN_QA_OUTPUT\models" --dry-run --raw
Confirm-ExitCode -Actual $LASTEXITCODE -Expected 0 -Test "Model download dry run"
```

Expected: structured metadata describes the selected files; the models directory remains empty. Remove `--dry-run` only with owner approval because model files can be large.

### NET-06 — Download a filtered remote bucket

```powershell
& $env:ACORN_BIN --no-local-database download https://github.com/jhwohlgemuth/bucket --filter 'index\.json$' --output "$env:ACORN_QA_OUTPUT\remote-bucket"
Confirm-ExitCode -Actual $LASTEXITCODE -Expected 0 -Test "Filtered remote bucket download"
```

Expected: matching research index files are downloaded without unrelated large content. A third-party repository move or outage should be recorded as `SKIP — external dependency`, not an ACORN pass.

### NET-07 — Preview provider enrichment

```powershell
& $env:ACORN_BIN --no-local-database format "$env:ACORN_QA_INPUT\valid-rad\index.json" --enrich --provider openalex,citeas --dry-run --no-color
Confirm-ExitCode -Actual $LASTEXITCODE -Expected 0 -Test "Metadata enrichment preview"
```

Expected: existing populated values remain intact and any provider additions appear only in the preview.

### NET-08 — Canonicalize and strictly ingest a remote bucket

```powershell
& $env:ACORN_BIN --database-path "$env:ACORN_QA_DATABASE" download https://github.com/jhwohlgemuth/bucket --filter 'index\.json$' --canonicalize --strict --output "$env:ACORN_QA_OUTPUT\canonical-bucket"
Confirm-ExitCode -Actual $LASTEXITCODE -Expected 0 -Test "Canonical, strict bucket ingestion"
```

Expected: deprecated aliases are rewritten before strict database ingestion. If the external test repository contains no deprecated aliases, the test still verifies that the combined mode completes.

## 11. Owner-supervised infrastructure tests

Do not give these steps to an unsupervised tester. They need real GitLab credentials and/or a container runtime and may change external state.

### INFRA-01 — Confirm create/serve argument validation without changing anything

```powershell
& $env:ACORN_BIN create runner --help *> $null
$runnerHelp = $LASTEXITCODE
& $env:ACORN_BIN create bot --help *> $null
$createBotHelp = $LASTEXITCODE
& $env:ACORN_BIN serve bot --help *> $null
$serveBotHelp = $LASTEXITCODE
if (@($runnerHelp, $createBotHelp, $serveBotHelp) | Where-Object { $_ -ne 0 }) { throw "[FAIL] Infrastructure help failed" }
Write-Host "[PASS] Runner and bot interfaces are available without side effects" -ForegroundColor Green
```

### INFRA-02 — Create a GitLab runner (owner performs substitutions)

```powershell
& $env:ACORN_BIN create runner --group '<GITLAB_GROUP_ID>' --name 'acorn-qa-runner' --runtime docker --repo '<GITLAB_DOMAIN>'
```

Expected: a runner is registered and its container starts. The owner must remove the runner and container after the test.

### INFRA-03 — Create and run a detached GitLab bot (owner performs substitutions)

```powershell
& $env:ACORN_BIN create bot '<GITLAB_PROJECT_ID>' --name 'acorn-qa-bot' --runtime docker --detach --domain '<GITLAB_DOMAIN>'
```

Expected: the container starts, authenticates, and polls without exposing secrets. The owner must remove the test bot/container afterward.

### INFRA-04 — Run the in-process bot service (owner performs substitutions)

```powershell
& $env:ACORN_BIN serve bot '<GITLAB_PROJECT_ID>' --event-source poll --poll-interval 10
```

Expected: the service starts, handles a test note event once, and stops cleanly with `Ctrl+C`.

Webhook and hybrid delivery should be tested only with an approved HTTPS test URL, project, webhook token, and signing token. Never expose a development machine directly to the public internet for this test.

## 12. Explicitly incomplete interfaces

These tests document intentional gaps. They pass when the CLI rejects the request clearly and non-destructively.

### GAP-01 — `create mcp` reports that it is not implemented

```powershell
& $env:ACORN_BIN --no-local-database create mcp
Confirm-NonzeroExitCode -Actual $LASTEXITCODE -Test "create mcp reports an implementation gap"
```

### MCP-01 — `serve mcp` starts the stdio protocol server

```powershell
@'{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"acceptance","version":"1"}}}'@ | & $env:ACORN_BIN --no-local-database serve mcp
Confirm-ExitCode -Actual $LASTEXITCODE -Expected 0 -Test "serve mcp protocol initialization"
```

### GAP-03 — Doctor autofix reports that it is not implemented

```powershell
& $env:ACORN_BIN --no-local-database doctor --fix
Confirm-NonzeroExitCode -Actual $LASTEXITCODE -Test "Doctor autofix reports an implementation gap"
```

## 13. Global behavior and safety

### GLOBAL-01 — Verify quiet and verbose modes

```powershell
$quiet = & $env:ACORN_BIN --offline --no-local-database gather --text "doi:10.1038/nature12373" --quiet 2>&1 | Out-String
$quietCode = $LASTEXITCODE
$verbose = & $env:ACORN_BIN --offline --no-local-database gather --text "doi:10.1038/nature12373" -vv 2>&1 | Out-String
$verboseCode = $LASTEXITCODE
if ($quietCode -ne 0 -or $verboseCode -ne 0) { throw "[FAIL] Quiet or verbose command failed" }
if ($quiet.Length -ge $verbose.Length) { throw "[FAIL] Quiet output was not smaller than verbose output" }
Write-Host "[PASS] Quiet and verbose levels produce appropriately different output" -ForegroundColor Green
```

### GLOBAL-02 — Verify thread-count parsing

```powershell
& $env:ACORN_BIN --offline --no-local-database --threads 1 gather --text "doi:10.1038/nature12373" --raw *> $null
Confirm-ExitCode -Actual $LASTEXITCODE -Expected 0 -Test "Single-thread mode"
```

### GLOBAL-03 — Reject conflicting database flags

```powershell
& $env:ACORN_BIN --database-path "$env:ACORN_QA_DATABASE" --no-local-database doctor
Confirm-NonzeroExitCode -Actual $LASTEXITCODE -Test "Conflicting database options are rejected"
```

### GLOBAL-04 — Request the optional DuckDB backend in isolation

```powershell
$duckdb = "$env:ACORN_QA_ROOT\database\acceptance.duckdb"
& $env:ACORN_BIN --offline --database-backend duckdb --database-path $duckdb gather --text "doi:10.1038/nature12373" --format json > "$env:ACORN_QA_OUTPUT\duckdb-gather.json" 2> "$env:ACORN_QA_OUTPUT\duckdb-error.txt"
$code = $LASTEXITCODE
if ($code -eq 0) {
    Confirm-Path -Path $duckdb -Test "DuckDB database was created"
} elseif (Select-String -LiteralPath "$env:ACORN_QA_OUTPUT\duckdb-error.txt" -Pattern "not available in this build" -Quiet) {
    Write-Host "[PASS] This build clearly reports that optional DuckDB support is unavailable" -ForegroundColor Green
} else {
    throw "[FAIL] DuckDB neither worked nor reported the expected feature-gate message"
}
```

### GLOBAL-05 — Reset only the disposable SQLite database

Run after GATHER-05 and before cleanup.

```powershell
$resolvedDatabase = [System.IO.Path]::GetFullPath($env:ACORN_QA_DATABASE)
$allowedDatabaseRoot = [System.IO.Path]::GetFullPath("$env:ACORN_QA_ROOT\database")
if (-not $resolvedDatabase.StartsWith($allowedDatabaseRoot, [System.StringComparison]::OrdinalIgnoreCase)) {
    throw "[FAIL] Refusing to reset a database outside the QA workspace"
}
& $env:ACORN_BIN --offline --database-path $resolvedDatabase --reset-database gather --text "doi:10.1038/nature12373" --format json > "$env:ACORN_QA_OUTPUT\after-reset.json"
Confirm-ExitCode -Actual $LASTEXITCODE -Expected 0 -Test "Disposable database reset"
```

Do not test `--clear-cache` against a normal user profile: it intentionally removes ACORN cache data and downloaded Chromium artifacts. Test `--reset-database` only against `$env:ACORN_QA_DATABASE` and only when earlier database tests no longer need their data.

## 14. Completion and cleanup

### CLEANUP-01 — Display the artifacts for review

```powershell
Get-ChildItem -LiteralPath $env:ACORN_QA_ROOT -Recurse | Select-Object FullName, Length
Write-Host "Review failures before cleanup. Workspace: $env:ACORN_QA_ROOT" -ForegroundColor Cyan
```

### CLEANUP-02 — Remove only this disposable run

Run only after screenshots and logs have been saved. This deletion cannot be undone.

```powershell
$resolved = (Resolve-Path -LiteralPath $env:ACORN_QA_ROOT).Path
$allowedRoot = (Resolve-Path -LiteralPath "$env:ACORN_REPO\target\qa-acceptance").Path
if (-not $resolved.StartsWith($allowedRoot, [System.StringComparison]::OrdinalIgnoreCase)) {
    throw "[FAIL] Refusing to delete an unexpected path: $resolved"
}
Remove-Item -LiteralPath $resolved -Recurse -Force
Write-Host "[PASS] Removed disposable acceptance workspace: $resolved" -ForegroundColor Green
```

## Release acceptance recommendation

A release candidate is acceptable when:

- every applicable test in sections 1–8 and 13 passes;
- PDF and PowerPoint artifacts pass visual inspection on at least one supported workstation;
- watch and TUI behavior pass on an interactive terminal build;
- connected tests pass in an environment authorized for network access;
- infrastructure tests pass only when those capabilities are part of the release scope;
- incomplete interfaces continue to fail clearly until they are implemented;
- no test writes outside the disposable workspace except the explicitly documented skill cache/clipboard and owner-supervised infrastructure actions;
- every failure has the test ID, version, operating system, terminal output, and retained artifact path.

Git branch, commit, merge-request, filter, and ignore selectors share the same source-resolution pipeline across `check`, `format`, `export`, and `link`. They should receive a short repository-specific smoke test when a release changes that pipeline. Merge-request tests require authenticated GitLab or GitHub context and belong with owner-supervised integration testing.
