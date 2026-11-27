@echo off
rem Builds the Windows program and packs it for deployment on a Windows PC.
rem Needs Rust for Windows (https://rustup.rs) with the MSVC build tools.
rem Output: dist\Tag-Rename-1.1.6-windows-x64.zip (program, README, checksum).
setlocal
cd /d "%~dp0"

cargo build --release
if errorlevel 1 (
    echo Build failed. See the messages above.
    exit /b 1
)

set NAME=Tag-Rename-1.1.6-windows-x64
set STAGE=dist\%NAME%
if exist "%STAGE%" rmdir /s /q "%STAGE%"
mkdir "%STAGE%"
copy /y "target\release\tagrename.exe" "%STAGE%\Tag-Rename.exe" >nul
copy /y "package\README-windows.txt" "%STAGE%\README.txt" >nul
copy /y "assets\tagrename.ico" "%STAGE%\tagrename.ico" >nul

powershell -NoProfile -Command "Compress-Archive -Path '%STAGE%\*' -DestinationPath 'dist\%NAME%.zip' -Force"
rmdir /s /q "%STAGE%"

echo Package written to dist\%NAME%.zip
endlocal
