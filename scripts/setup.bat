@echo off

scoop install vcpkg
vcpkg install openssl:x64-windows
vcpkg install openssl:x64-windows-static

SET OPENSSL_DIR=%USERPROFILE%\scoop\apps\vcpkg\current\installed\x64-windows-static