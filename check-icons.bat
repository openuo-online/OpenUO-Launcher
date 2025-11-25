@echo off
chcp 65001 >nul
echo [Checking Icon Configuration...]
echo.

REM 检查图标文件
echo [1. Checking icon files...]
if exist "assets\icon.ico" (
    echo    ✓ assets\icon.ico exists
    for %%A in (assets\icon.ico) do echo      Size: %%~zA bytes
) else (
    echo    ✗ assets\icon.ico NOT FOUND!
    echo      This is required for Windows .exe icon
)

if exist "assets\icon.icns" (
    echo    ✓ assets\icon.icns exists
    for %%A in (assets\icon.icns) do echo      Size: %%~zA bytes
) else (
    echo    ✗ assets\icon.icns NOT FOUND!
    echo      This is required for macOS .app icon
)

if exist "assets\logo.png" (
    echo    ✓ assets\logo.png exists
    for %%A in (assets\logo.png) do echo      Size: %%~zA bytes
) else (
    echo    ✗ assets\logo.png NOT FOUND!
    echo      This is required for window icon
)

echo.
echo [2. Checking build.rs configuration...]
findstr /C:"icon.ico" build.rs >nul
if %errorlevel% equ 0 (
    echo    ✓ build.rs references icon.ico
) else (
    echo    ✗ build.rs does NOT reference icon.ico
)

echo.
echo [3. Checking Cargo.toml dependencies...]
findstr /C:"winres" Cargo.toml >nul
if %errorlevel% equ 0 (
    echo    ✓ winres dependency found
) else (
    echo    ✗ winres dependency NOT FOUND
)

echo.
echo [4. Checking installer configurations...]
if exist "installer.iss" (
    echo    ✓ installer.iss exists (Inno Setup)
    findstr /C:"icon.ico" installer.iss >nul
    if %errorlevel% equ 0 (
        echo      ✓ References icon.ico
    ) else (
        echo      ✗ Does NOT reference icon.ico
    )
) else (
    echo    ✗ installer.iss NOT FOUND
)

if exist "installer.wxs" (
    echo    ✓ installer.wxs exists (WiX)
    findstr /C:"icon.ico" installer.wxs >nul
    if %errorlevel% equ 0 (
        echo      ✓ References icon.ico
    ) else (
        echo      ✗ Does NOT reference icon.ico
    )
) else (
    echo    ✗ installer.wxs NOT FOUND
)

echo.
echo [5. Checking compiled executable (if exists)...]
if exist "target\release\openuo-launcher.exe" (
    echo    ✓ target\release\openuo-launcher.exe exists
    echo      To verify icon is embedded, right-click the file and check Properties
) else (
    echo    ⚠ target\release\openuo-launcher.exe not built yet
    echo      Run 'cargo build --release' to build
)

echo.
echo [Summary]
echo ========================================
echo Icon configuration check complete!
echo.
echo Next steps:
echo 1. If any files are missing, create them
echo 2. Run 'cargo build --release' to build with icons
echo 3. Check the .exe properties to verify icon is embedded
echo 4. Run build-windows-installer.bat to create installer
echo.
pause
