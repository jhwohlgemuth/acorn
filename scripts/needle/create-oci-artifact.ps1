[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [string]$SourceDirectory,
    [Parameter(Mandatory = $true)]
    [string]$Reference,
    [string]$BundleDirectory = "needle-modelkit",
    [ValidateSet("windows-x86_64", "windows-arm64", "linux-x86_64", "linux-arm64", "macos-arm64")]
    [string]$Platform,
    [string]$Acorn = "acorn",
    [string]$Oras = "oras",
    [string]$SourceRevision = "b274efcb211a9eef48c9a88da4b43bd569696a39",
    [string]$RegistryConfig,
    [string]$CaFile,
    [switch]$PlainHttp
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

if (-not (Test-Path -LiteralPath $SourceDirectory -PathType Container)) {
    throw "Needle 3 source directory does not exist: $SourceDirectory"
}
if (Test-Path -LiteralPath $BundleDirectory) {
    throw "Refusing to overwrite bundle path: $BundleDirectory"
}
$acornCommand = Get-Command $Acorn -ErrorAction SilentlyContinue
if (-not $acornCommand) {
    throw "ACORN executable is unavailable: $Acorn"
}
$orasCommand = Get-Command $Oras -ErrorAction SilentlyContinue
if (-not $orasCommand) {
    throw "ORAS executable is unavailable: $Oras"
}
$acornExecutable = $acornCommand.Source
$orasExecutable = $orasCommand.Source
if ($SourceRevision -notmatch '^[0-9a-fA-F]{40}$') {
    throw "SourceRevision must be a 40-character hexadecimal revision"
}

if ([string]::IsNullOrWhiteSpace($Platform)) {
    $os = if ([System.Runtime.InteropServices.RuntimeInformation]::IsOSPlatform([System.Runtime.InteropServices.OSPlatform]::Windows)) {
        "windows"
    } elseif ([System.Runtime.InteropServices.RuntimeInformation]::IsOSPlatform([System.Runtime.InteropServices.OSPlatform]::Linux)) {
        "linux"
    } elseif ([System.Runtime.InteropServices.RuntimeInformation]::IsOSPlatform([System.Runtime.InteropServices.OSPlatform]::OSX)) {
        "macos"
    } else {
        throw "Cannot infer a supported Needle operating system"
    }
    $architecture = [System.Runtime.InteropServices.RuntimeInformation]::ProcessArchitecture.ToString().ToLowerInvariant()
    $architecture = switch ($architecture) {
        "x64" { "x86_64" }
        "arm64" { "arm64" }
        default { throw "Cannot infer a supported Needle architecture: $architecture" }
    }
    $Platform = "$os-$architecture"
}
$supportedPlatforms = @("windows-x86_64", "windows-arm64", "linux-x86_64", "linux-arm64", "macos-arm64")
if ($Platform -notin $supportedPlatforms) {
    throw "Unsupported Needle platform: $Platform"
}

$runnerName = if ($Platform.StartsWith("windows-")) { "needle.exe" } else { "needle" }
$runnerSource = Join-Path $SourceDirectory $runnerName
$modelSource = Join-Path $SourceDirectory "needle3.cact"
$licenseSource = Join-Path $SourceDirectory "LICENSE"
foreach ($requiredFile in @($runnerSource, $modelSource, $licenseSource)) {
    if (-not (Test-Path -LiteralPath $requiredFile -PathType Leaf)) {
        throw "Needle 3 bundle input does not exist: $requiredFile"
    }
}
$registryReference = $Reference -replace '^oci://', ''
if ($registryReference.Contains('@')) {
    throw "Publish to a tag, not an existing digest: $Reference"
}

& $acornExecutable export --tools --output $BundleDirectory
if ($LASTEXITCODE -ne 0) { throw "ACORN failed to export tools.json" }
Copy-Item -LiteralPath $runnerSource -Destination (Join-Path $BundleDirectory $runnerName)
Copy-Item -LiteralPath $modelSource -Destination (Join-Path $BundleDirectory "needle3.cact")
Copy-Item -LiteralPath $licenseSource -Destination (Join-Path $BundleDirectory "LICENSE")
$licensePath = Join-Path $BundleDirectory "LICENSE"
$modelPath = Join-Path $BundleDirectory "needle3.cact"
$runnerPath = Join-Path $BundleDirectory $runnerName
$toolsPath = Join-Path $BundleDirectory "tools.json"
$licenseSha256 = (Get-FileHash -LiteralPath $licensePath -Algorithm SHA256).Hash.ToLowerInvariant()
$licenseSize = (Get-Item -LiteralPath $licensePath).Length
$modelSha256 = (Get-FileHash -LiteralPath $modelPath -Algorithm SHA256).Hash.ToLowerInvariant()
$modelSize = (Get-Item -LiteralPath $modelPath).Length
$runnerSha256 = (Get-FileHash -LiteralPath $runnerPath -Algorithm SHA256).Hash.ToLowerInvariant()
$runnerSize = (Get-Item -LiteralPath $runnerPath).Length
$toolsSha256 = (Get-FileHash -LiteralPath $toolsPath -Algorithm SHA256).Hash.ToLowerInvariant()
$toolsSize = (Get-Item -LiteralPath $toolsPath).Length
$manifest = [ordered]@{
    manifestVersion = "v1.0.0"
    sourceRevision = $SourceRevision
    platform = $Platform
    sidecar = [ordered]@{
        kind = "needle"
        protocolVersion = 2
    }
    runner = [ordered]@{
        path = $runnerName
        size = $runnerSize
        sha256 = $runnerSha256
    }
    assets = @(
        [ordered]@{
            name = "license"
            path = "LICENSE"
            size = $licenseSize
            sha256 = $licenseSha256
        },
        [ordered]@{
            name = "model"
            path = "needle3.cact"
            size = $modelSize
            sha256 = $modelSha256
        },
        [ordered]@{
            name = "tools"
            path = "tools.json"
            size = $toolsSize
            sha256 = $toolsSha256
        }
    )
    tools = "tools"
} | ConvertTo-Json -Depth 4
$manifestPath = Join-Path $BundleDirectory "needle-modelkit.json"
[System.IO.File]::WriteAllText($manifestPath, $manifest, [System.Text.UTF8Encoding]::new($false))

$transportArguments = @()
if (-not [string]::IsNullOrWhiteSpace($RegistryConfig)) { $transportArguments += @("--registry-config", $RegistryConfig) }
if (-not [string]::IsNullOrWhiteSpace($CaFile)) { $transportArguments += @("--ca-file", $CaFile) }
if ($PlainHttp) { $transportArguments += "--plain-http" }

Push-Location $BundleDirectory
try {
    & $orasExecutable push `
        --artifact-type application/vnd.acorn.needle.modelkit.v2 `
        $registryReference `
        "needle-modelkit.json:application/vnd.acorn.needle.modelkit.manifest.v2+json" `
        "LICENSE:text/plain" `
        "needle3.cact:application/vnd.acorn.needle.model.v3" `
        "tools.json:application/json" `
        "${runnerName}:application/vnd.acorn.needle.runner.v3" `
        @transportArguments
    if ($LASTEXITCODE -ne 0) { throw "ORAS failed to push the Needle artifact" }
} finally {
    Pop-Location
}

$resolvedReference = (& $orasExecutable resolve --full-reference $registryReference @transportArguments | Select-Object -Last 1).Trim()
if ($LASTEXITCODE -ne 0) { throw "ORAS failed to resolve the pushed artifact" }
if ($resolvedReference -notmatch '@sha256:[0-9a-fA-F]{64}$') {
    throw "ORAS returned an unexpected resolved reference: $resolvedReference"
}

Write-Output "Bundle: $BundleDirectory"
Write-Output "Custom location: oci://$resolvedReference"
Write-Output "Run: acorn infer `"Return the ACORN version`" --custom-location oci://$resolvedReference"
