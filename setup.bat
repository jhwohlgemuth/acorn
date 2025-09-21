scoop install vcpkg
vcpkg install openssl:x64-windows
vcpkg install openssl:x64-windows-static
$Env:OPENSSL_DIR="${Env:USERPROFILE}\scoop\apps\vcpkg\current\installed\x64-windows-static"