@echo off
chcp 65001 > nul
echo =======================================================================
echo     VCKey 2.0.5 Pro Native (Tauri v2 + Rust)
echo =======================================================================
echo.
echo [*] Dang khoi dong VCKey Native...
start "" "%~dp0src-tauri\target\release\vckey-tauri.exe"
exit
