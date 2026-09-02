[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [string]$Runner,
    [Parameter(Mandatory = $true)]
    [string]$Reference,
    [string]$BundleDirectory = "needle-modelkit",
    [ValidateSet("windows-x86_64", "windows-arm64", "linux-x86_64", "linux-arm64", "macos-arm64")]
    [string]$Platform,
    [string]$Acorn = "acorn",
    [string]$Oras = "oras",
    [string]$SourceRevision = "98fbd955b0347e78059be0c253cc1ffa09b87bc7",
    [string]$RegistryConfig,
    [string]$CaFile,
    [switch]$PlainHttp
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

if (-not (Test-Path -LiteralPath $Runner -PathType Leaf)) {
    throw "Needle runner does not exist: $Runner"
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
$registryReference = $Reference -replace '^oci://', ''
if ($registryReference.Contains('@')) {
    throw "Publish to a tag, not an existing digest: $Reference"
}

& $acornExecutable export --tools --output $BundleDirectory
if ($LASTEXITCODE -ne 0) { throw "ACORN failed to export tools.json" }
Copy-Item -LiteralPath $Runner -Destination (Join-Path $BundleDirectory $runnerName)
$runnerSha256 = (Get-FileHash -LiteralPath (Join-Path $BundleDirectory $runnerName) -Algorithm SHA256).Hash.ToLowerInvariant()
$manifest = [ordered]@{
    schemaVersion = 1
    sourceRevision = $SourceRevision
    platform = $Platform
    runner = $runnerName
    runnerSha256 = $runnerSha256
    tools = "tools.json"
} | ConvertTo-Json
$manifestPath = Join-Path $BundleDirectory "needle-modelkit.json"
[System.IO.File]::WriteAllText($manifestPath, $manifest, [System.Text.UTF8Encoding]::new($false))

$transportArguments = @()
if (-not [string]::IsNullOrWhiteSpace($RegistryConfig)) { $transportArguments += @("--registry-config", $RegistryConfig) }
if (-not [string]::IsNullOrWhiteSpace($CaFile)) { $transportArguments += @("--ca-file", $CaFile) }
if ($PlainHttp) { $transportArguments += "--plain-http" }

Push-Location $BundleDirectory
try {
    & $orasExecutable push `
        --artifact-type application/vnd.acorn.needle.modelkit.v1 `
        $registryReference `
        "needle-modelkit.json:application/vnd.acorn.needle.modelkit.manifest.v1+json" `
        "tools.json:application/json" `
        "${runnerName}:application/vnd.acorn.needle.runner" `
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
