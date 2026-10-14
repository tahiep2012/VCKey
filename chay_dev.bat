@echo off
chcp 65001 > nul
echo =======================================================================
echo     VCKey 2.0.5 - Chế Độ Chạy Thử Nhanh (Dev Mode)
echo =======================================================================
echo.
echo [*] Dang bien dich gia toc va khoi chay VCKey truc tiep tu ma nguon...
cd /d "%~dp0src-tauri"
set "PATH=C:\Users\Elt.pur02\AppData\Local\Microsoft\WinGet\Packages\MartinStorsjo.LLVM-MinGW.UCRT_Microsoft.Winget.Source_8wekyb3d8bbwe\llvm-mingw-20260616-ucrt-x86_64\bin;C:\Users\Elt.pur02\.cargo\bin;%PATH%"
cargo run
