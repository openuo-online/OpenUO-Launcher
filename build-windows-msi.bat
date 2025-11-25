@echo off
chcp 65001 >nul
setlocal enabledelayedexpansion

echo [Building OpenUO Launcher Windows MSI Installer...]
echo.

REM 检查 WiX Toolset 是否安装
set WIX_PATH=
if exist "C:\Program Files (x86)\WiX Toolset v3.11\bin\candle.exe" (
    set WIX_PATH=C:\Program Files (x86)\WiX Toolset v3.11\bin
) else if exist "C:\Program Files\WiX Toolset v3.11\bin\candle.exe" (
    set WIX_PATH=C:\Program Files\WiX Toolset v3.11\bin
) else if exist "%WIX%bin\candle.exe" (
    set WIX_PATH=%WIX%bin
) else (
    echo [Error] WiX Toolset not found!
    echo [Please download and install from: https://wixtoolset.org/releases/]
    echo.
    pause
    exit /b 1
)

echo [Found WiX Toolset: %WIX_PATH%]
echo.

REM 读取版本号
for /f "tokens=2 delims==" %%a in ('findstr /r "^version" Cargo.toml') do (
    set VERSION=%%a
    set VERSION=!VERSION:"=!
    set VERSION=!VERSION: =!
)

echo [Version: %VERSION%]
echo.

REM 构建 release 版本
echo [Building release binary...]
cargo build --release
if errorlevel 1 (
    echo [Build failed!]
    exit /b 1
)

echo.
echo [Compiling WiX source...]

REM 创建输出目录
if not exist releases mkdir releases
if not exist build mkdir build

REM 编译 .wxs 到 .wixobj
"%WIX_PATH%\candle.exe" -dProductVersion=%VERSION% -out build\ installer.wxs
if errorlevel 1 (
    echo [WiX compilation failed!]
    exit /b 1
)

echo.
echo [Linking MSI installer...]

REM 链接生成 MSI
"%WIX_PATH%\light.exe" -ext WixUIExtension -out "releases\OpenUO-Launcher-v%VERSION%.msi" build\installer.wixobj
if errorlevel 1 (
    echo [MSI linking failed!]
    exit /b 1
)

echo.
echo [✓] Build complete!
echo [Location: releases\OpenUO-Launcher-v%VERSION%.msi]
echo.

REM 显示文件大小
for %%A in (releases\OpenUO-Launcher-v%VERSION%.msi) do (
    set SIZE=%%~zA
    set /a SIZE_MB=!SIZE! / 1048576
    echo [File size: !SIZE_MB! MB]
)

echo.
echo [Done!]
pause
