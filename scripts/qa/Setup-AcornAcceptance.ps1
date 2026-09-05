[CmdletBinding()]
param(
    [string]$AcornPath
)

$ErrorActionPreference = "Stop"

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..")).Path
$candidate = if ($AcornPath) {
    $AcornPath
} elseif (Test-Path (Join-Path $repoRoot "target\debug\acorn.exe")) {
    Join-Path $repoRoot "target\debug\acorn.exe"
} else {
    "acorn"
}
$command = Get-Command $candidate -ErrorAction Stop
$binary = if ($command.CommandType -eq "Application") {
    $command.Source
} else {
    throw "ACORN must resolve to an executable application. Received: $($command.CommandType)"
}

$stamp = Get-Date -Format "yyyyMMdd-HHmmss"
$qaRoot = Join-Path $repoRoot "target\qa-acceptance\$stamp"
$inputRoot = Join-Path $qaRoot "input"
$outputRoot = Join-Path $qaRoot "output"
$configRoot = Join-Path $qaRoot "config"
$databaseRoot = Join-Path $qaRoot "database"
$null = New-Item -ItemType Directory -Force -Path $inputRoot, $outputRoot, $configRoot, $databaseRoot

$fixtureRoot = Join-Path $repoRoot "tests\fixtures"
Copy-Item (Join-Path $fixtureRoot "data\valid_project") (Join-Path $inputRoot "valid-rad") -Recurse
Copy-Item (Join-Path $fixtureRoot "data\valid_project") (Join-Path $inputRoot "bag-source") -Recurse
Copy-Item (Join-Path $fixtureRoot "data\format\changes") (Join-Path $inputRoot "format-rad") -Recurse
Copy-Item (Join-Path $fixtureRoot "data\format\changes") (Join-Path $inputRoot "link-rad") -Recurse
Copy-Item (Join-Path $fixtureRoot "data\highlight\hecate") (Join-Path $inputRoot "highlight") -Recurse
Copy-Item (Join-Path $fixtureRoot "data\highlight\reference.pptx") (Join-Path $inputRoot "highlight\reference.pptx")
Copy-Item (Join-Path $fixtureRoot "schema") (Join-Path $inputRoot "schema") -Recurse
Copy-Item (Join-Path $fixtureRoot "gather") (Join-Path $inputRoot "gather") -Recurse
Copy-Item (Join-Path $fixtureRoot "sample.docx") (Join-Path $inputRoot "sample.docx")
Copy-Item (Join-Path $fixtureRoot "CITATION.cff") (Join-Path $inputRoot "CITATION.cff")
Copy-Item (Join-Path $fixtureRoot "malformed-rad.json") (Join-Path $inputRoot "malformed-rad.json")
Copy-Item (Join-Path $fixtureRoot "data\invalid_project_b\index.yaml") (Join-Path $inputRoot "rad.yaml")
Copy-Item (Join-Path $fixtureRoot "data\invalid_project_c\index.md") (Join-Path $inputRoot "rad.md")
Copy-Item (Join-Path $fixtureRoot "raid\success.json") (Join-Path $inputRoot "raid.json")
Copy-Item (Join-Path $fixtureRoot "config\basic.jsonc") (Join-Path $configRoot "basic.jsonc")

@"
openapi: 3.0.3
info:
  title: ACORN QA API
  version: 1.0.0
servers:
  - url: https://api.example.org/v1
paths:
  /projects:
    get:
      operationId: listProjects
      responses:
        '200':
          description: Success
"@ | Set-Content -LiteralPath (Join-Path $inputRoot "openapi.yaml") -Encoding utf8

@"
acme/example-model
"@ | Set-Content -LiteralPath (Join-Path $inputRoot "models.txt") -Encoding utf8

@"
buckets:
  - name: local
    repository:
      provider: git
      location: file:./tests/fixtures/data/bucket/
"@ | Set-Content -LiteralPath (Join-Path $configRoot "local.yaml") -Encoding utf8

@"
This acceptance-test file contains DOI 10.1038/nature12373, ORCID 0000-0002-1825-0097,
ROR https://ror.org/01qz5mb56, and arXiv:2101.00001.
"@ | Set-Content -LiteralPath (Join-Path $inputRoot "identifiers.txt") -Encoding utf8

$env:ACORN_BIN = $binary
$env:ACORN_REPO = $repoRoot
$env:ACORN_QA_ROOT = $qaRoot
$env:ACORN_QA_INPUT = $inputRoot
$env:ACORN_QA_OUTPUT = $outputRoot
$env:ACORN_QA_CONFIG = $configRoot
$env:ACORN_QA_DATABASE = Join-Path $databaseRoot "acceptance.sqlite"
$global:AcornQaPreviousDatabasePath = $env:ACORN_DATABASE_PATH
Remove-Item Env:ACORN_DATABASE_PATH -ErrorAction SilentlyContinue
$chromeCandidates = @(
    $env:CHROME_PATH,
    (Join-Path $env:ProgramFiles "Google\Chrome\Application\chrome.exe"),
    (Join-Path ${env:ProgramFiles(x86)} "Microsoft\Edge\Application\msedge.exe"),
    (Join-Path $env:LOCALAPPDATA "Google\Chrome\Application\chrome.exe")
) | Where-Object { $_ -and (Test-Path -LiteralPath $_) }
$env:ACORN_QA_CHROME = $chromeCandidates | Select-Object -First 1

function Confirm-ExitCode {
    param(
        [Parameter(Mandatory)] [int]$Actual,
        [Parameter(Mandatory)] [int]$Expected,
        [Parameter(Mandatory)] [string]$Test
    )
    if ($Actual -ne $Expected) {
        throw "[FAIL] $Test (expected exit code $Expected, received $Actual)"
    }
    Write-Host "[PASS] $Test" -ForegroundColor Green
}

function Confirm-NonzeroExitCode {
    param(
        [Parameter(Mandatory)] [int]$Actual,
        [Parameter(Mandatory)] [string]$Test
    )
    if ($Actual -eq 0) {
        throw "[FAIL] $Test (expected a nonzero exit code)"
    }
    Write-Host "[PASS] $Test" -ForegroundColor Green
}

function Confirm-Path {
    param(
        [Parameter(Mandatory)] [string]$Path,
        [Parameter(Mandatory)] [string]$Test
    )
    if (-not (Test-Path -LiteralPath $Path)) {
        throw "[FAIL] $Test (missing $Path)"
    }
    Write-Host "[PASS] $Test" -ForegroundColor Green
}

function Confirm-Text {
    param(
        [Parameter(Mandatory)] [string]$Path,
        [Parameter(Mandatory)] [string]$Pattern,
        [Parameter(Mandatory)] [string]$Test
    )
    if (-not (Select-String -LiteralPath $Path -Pattern $Pattern -Quiet)) {
        throw "[FAIL] $Test (did not find '$Pattern' in $Path)"
    }
    Write-Host "[PASS] $Test" -ForegroundColor Green
}

Write-Host "ACORN acceptance workspace is ready." -ForegroundColor Cyan
Write-Host "Binary:   $env:ACORN_BIN"
Write-Host "Inputs:   $env:ACORN_QA_INPUT"
Write-Host "Outputs:  $env:ACORN_QA_OUTPUT"
Write-Host "Database: $env:ACORN_QA_DATABASE"
Write-Host "Browser:  $(if ($env:ACORN_QA_CHROME) { $env:ACORN_QA_CHROME } else { 'not detected (PDF test is optional)' })"
Write-Host "Use this PowerShell window for every test step; the variables and helper functions live in this session."
