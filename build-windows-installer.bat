@echo off
chcp 65001 >nul
setlocal enabledelayedexpansion

echo [Building OpenUO Launcher Windows Installer...]
echo.

REM 检查 Inno Setup 是否安装
set ISCC_PATH=
if exist "C:\Program Files (x86)\Inno Setup 6\ISCC.exe" (
    set ISCC_PATH=C:\Program Files (x86)\Inno Setup 6\ISCC.exe
) else if exist "C:\Program Files\Inno Setup 6\ISCC.exe" (
    set ISCC_PATH=C:\Program Files\Inno Setup 6\ISCC.exe
) else (
    echo [Error] Inno Setup not found!
    echo [Please download and install from: https://jrsoftware.org/isdl.php]
    echo.
    pause
    exit /b 1
)

echo [Found Inno Setup: %ISCC_PATH%]
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
echo [Creating installer...]

REM 创建输出目录
if not exist releases mkdir releases

REM 更新 installer.iss 中的版本号
powershell -Command "(Get-Content installer.iss) -replace '#define MyAppVersion \".*\"', '#define MyAppVersion \"%VERSION%\"' | Set-Content installer.iss"

REM 编译安装程序
"%ISCC_PATH%" installer.iss
if errorlevel 1 (
    echo [Installer creation failed!]
    exit /b 1
)

echo.
echo [✓] Build complete!
echo [Location: releases\OpenUO-Launcher-Setup-v%VERSION%.exe]
echo.

REM 显示文件大小
for %%A in (releases\OpenUO-Launcher-Setup-v%VERSION%.exe) do (
    set SIZE=%%~zA
    set /a SIZE_MB=!SIZE! / 1048576
    echo [File size: !SIZE_MB! MB]
)

echo.
echo [Done!]
pause
