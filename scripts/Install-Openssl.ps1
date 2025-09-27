#Requires -Modules Prelude

Param()
function Install-Openssl() {
    vcpkg install openssl:x64-windows
    vcpkg install openssl:x64-windows-static
    $Env:OPENSSL_DIR="${Env:USERPROFILE}\scoop\apps\vcpkg\current\installed\x64-windows-static"
}
if (Test-Command -Name vcpkg) {
    Install-Openssl
} else {
    if (Test-Command -Name scoop) {
        scoop install vcpkg
    }
    Install-Openssl
}