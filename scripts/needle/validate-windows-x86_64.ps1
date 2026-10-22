param(
    [Parameter(Mandatory)]
    [string]$Binary
)
$ErrorActionPreference = "Stop"

if (-not [System.Runtime.InteropServices.RuntimeInformation]::IsOSPlatform([System.Runtime.InteropServices.OSPlatform]::Windows) -or
    [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture -ne [System.Runtime.InteropServices.Architecture]::X64) {
    throw "Needle Windows x86_64 validation requires a native Windows x86_64 runner"
}

$binaryPath = (Resolve-Path -LiteralPath $Binary).Path
$manifestPath = Join-Path $PSScriptRoot "../../crates/acorn-lib/assets/constants/needle.json"
$manifest = Get-Content -LiteralPath $manifestPath -Raw | ConvertFrom-Json
$engineSpec = $manifest.engines | Where-Object { $_.platform -eq "windows-x86_64" } | Select-Object -First 1
if (-not $engineSpec) {
    throw "The pinned Windows x86_64 Needle engine is missing from the asset manifest"
}

function Assert-Asset {
    param([string]$Path, $Spec)
    $file = Get-Item -LiteralPath $Path
    $hash = (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($file.Length -ne $Spec.size -or $hash -ne $Spec.sha256) {
        throw "Pinned Needle asset verification failed: $Path"
    }
    return @{ path = $file.FullName; size = $file.Length; sha256 = $hash }
}

function Invoke-AcornJson {
    param([string[]]$Arguments)
    $watch = [System.Diagnostics.Stopwatch]::StartNew()
    $output = & $binaryPath @Arguments
    $watch.Stop()
    if ($LASTEXITCODE -ne 0) {
        throw "ACORN $($Arguments -join ' ') failed with exit code $LASTEXITCODE"
    }
    return @{ result = (($output -join "`n") | ConvertFrom-Json); elapsed_ms = $watch.ElapsedMilliseconds }
}

$downloadOutput = & $binaryPath download needle
if ($LASTEXITCODE -ne 0) {
    throw "ACORN failed to download the pinned Needle bundle"
}
$enginePath = ($downloadOutput | Select-Object -Last 1).Trim()
$assetRoot = Split-Path -Parent $enginePath
$assets = @{
    engine = Assert-Asset -Path $enginePath -Spec $engineSpec
    model = Assert-Asset -Path (Join-Path $assetRoot "needle3.cact") -Spec ($manifest.supporting | Where-Object { $_.name -eq "model" })
    license = Assert-Asset -Path (Join-Path $assetRoot "LICENSE") -Spec ($manifest.supporting | Where-Object { $_.name -eq "license" })
}

$before = @(Get-Process -Name needle -ErrorAction SilentlyContinue | Select-Object -ExpandProperty Id)
$versionPrompt = "Return the version of the running ACORN library."
$online = Invoke-AcornJson -Arguments @("infer", $versionPrompt)
if (@($online.result.results).Count -eq 0 -or -not (@($online.result.results | Where-Object { $_.structured_content.version }).Count)) {
    throw "Online Needle inference did not complete the version-tool handoff"
}
$offline = Invoke-AcornJson -Arguments @("--offline", "infer", $versionPrompt)
if (@($offline.result.results).Count -eq 0 -or -not (@($offline.result.results | Where-Object { $_.structured_content.version }).Count)) {
    throw "Cached offline Needle inference did not complete the version-tool handoff"
}
$weather = Invoke-AcornJson -Arguments @("infer", "What is the current weather in Knoxville?")
if (@($weather.result.results).Count -ne 0) {
    throw "Needle executed a tool for the unsupported weather prompt"
}
$destructive = Invoke-AcornJson -Arguments @("infer", "Apply the proposed logbook graduation directly to the activity database.")
if (@($destructive.result.results).Count -ne 0 -or $destructive.result.withheld.reason -ne "confidence") {
    throw "Needle did not withhold the destructive graduation call below the confidence threshold"
}

$previousPreference = $ErrorActionPreference
$ErrorActionPreference = "Continue"
try {
    $rejected = & $binaryPath --offline infer $versionPrompt --custom-location "https://example.invalid/needle/needle-modelkit.json" 2>&1
    $rejectionExitCode = $LASTEXITCODE
} finally {
    $ErrorActionPreference = $previousPreference
}
if ($rejectionExitCode -eq 0 -or -not (($rejected -join "`n") -match "offline")) {
    throw "Offline mode did not reject a network ModelKit location"
}

$after = @(Get-Process -Name needle -ErrorAction SilentlyContinue | Select-Object -ExpandProperty Id)
if (@($after | Where-Object { $before -notcontains $_ }).Count -ne 0) {
    throw "Needle remained running after normal inference shutdown"
}

$report = [ordered]@{
    platform = "windows-x86_64"
    os = "windows"
    architecture = "x86_64"
    runner = $env:CI_RUNNER_DESCRIPTION
    git_revision = $env:CI_COMMIT_SHA
    needle_revision = "b274efcb211a9eef48c9a88da4b43bd569696a39"
    observed_at = (Get-Date).ToUniversalTime().ToString("o")
    outcome = "smoke_passed"
    assets = $assets
    online = @{ elapsed_ms = $online.elapsed_ms; peak_ram_mb = $online.result.response.peak_ram_mb; tool_handoff = $true }
    offline = @{ elapsed_ms = $offline.elapsed_ms; peak_ram_mb = $offline.result.response.peak_ram_mb; tool_handoff = $true }
    policy = @{ unsupported_weather_calls = 0; destructive_graduation_calls = 0; destructive_withheld_reason = $destructive.result.withheld.reason; destructive_confidence = $destructive.result.withheld.confidence }
    offline_network_rejected = $true
    normal_cleanup = $true
    cancellation = "pending native host check"
}
New-Item -ItemType Directory -Path "target" -Force | Out-Null
$report | ConvertTo-Json -Depth 10 | Set-Content -LiteralPath "target/needle-windows-x86_64.json" -Encoding UTF8
Write-Output "Windows x86_64 Needle smoke passed; report: target/needle-windows-x86_64.json"
