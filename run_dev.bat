@echo off
title VCKey - Test Nhanh (Debug Mode)
echo ========================================================
echo   VCKey - Dang khoi dong che do Test nhanh (Debug)...
echo ========================================================

taskkill /F /IM vckey-tauri.exe /IM VCKey.exe >nul 2>&1

set PATH=C:\Users\Elt.pur02\AppData\Local\Microsoft\WinGet\Packages\MartinStorsjo.LLVM-MinGW.UCRT_Microsoft.Winget.Source_8wekyb3d8bbwe\llvm-mingw-20260616-ucrt-x86_64\bin;C:\Users\Elt.pur02\.cargo\bin;%PATH%

cd /d "%~dp0src-tauri"
cargo run
