@echo off
chcp 65001 >nul
setlocal enabledelayedexpansion

echo [Building OpenUO Launcher...]
echo.

REM 读取版本号
for /f "tokens=2 delims==" %%a in ('findstr /r "^version" Cargo.toml') do (
    set VERSION=%%a
    set VERSION=!VERSION:"=!
    set VERSION=!VERSION: =!
)

echo [Version: %VERSION%]
echo.

REM 创建输出目录
if not exist releases mkdir releases

REM 检测平台
echo [Platform: Windows x64]
echo.

REM 构建
echo [Building release...]
cargo build --release
if errorlevel 1 (
    echo [Build failed!]
    exit /b 1
)

REM 复制到 releases 目录
set OUTPUT=releases\OpenUO-Launcher-windows-x64-v%VERSION%.exe
copy target\release\openuo-launcher.exe "%OUTPUT%"

echo.
echo [Build complete: %OUTPUT%]
echo.

REM 显示文件大小
for %%A in ("%OUTPUT%") do (
    set SIZE=%%~zA
    set /a SIZE_MB=!SIZE! / 1048576
    echo [File size: !SIZE_MB! MB]
)

echo.
echo [Done!]
