param(
    [Parameter(Mandatory)]
    [string]$Binary,
    [string]$Objdump = "objdump"
)
$ErrorActionPreference = "Stop"
$headers = & $Objdump -p $Binary
if ($LASTEXITCODE -ne 0) {
    throw "Could not inspect Windows binary: $Binary"
}
if (-not ($headers -match 'file format pei-x86-64')) {
    throw "Expected an x86_64 Windows executable: $Binary"
}
$runtimeImports = @($headers | Select-String 'DLL Name:\s*(libstdc\+\+|libgcc_s|libwinpthread)[^\s]*\.dll')
if ($runtimeImports.Count -gt 0) {
    throw "Windows releases must not depend on MinGW runtime DLLs: $($runtimeImports -join ', ')"
}
Write-Output "Windows runtime dependency check passed: $Binary"
