#!/bin/bash

echo "[Checking Icon Configuration...]"
echo

# 检查图标文件
echo "[1. Checking icon files...]"
if [ -f "assets/icon.ico" ]; then
    SIZE=$(ls -lh assets/icon.ico | awk '{print $5}')
    echo "   ✓ assets/icon.ico exists (Size: $SIZE)"
else
    echo "   ✗ assets/icon.ico NOT FOUND!"
    echo "     This is required for Windows .exe icon"
fi

if [ -f "assets/icon.icns" ]; then
    SIZE=$(ls -lh assets/icon.icns | awk '{print $5}')
    echo "   ✓ assets/icon.icns exists (Size: $SIZE)"
else
    echo "   ✗ assets/icon.icns NOT FOUND!"
    echo "     This is required for macOS .app icon"
fi

if [ -f "assets/logo.png" ]; then
    SIZE=$(ls -lh assets/logo.png | awk '{print $5}')
    echo "   ✓ assets/logo.png exists (Size: $SIZE)"
else
    echo "   ✗ assets/logo.png NOT FOUND!"
    echo "     This is required for window icon"
fi

echo
echo "[2. Checking build.rs configuration...]"
if grep -q "icon.ico" build.rs 2>/dev/null; then
    echo "   ✓ build.rs references icon.ico"
else
    echo "   ✗ build.rs does NOT reference icon.ico"
fi

echo
echo "[3. Checking Cargo.toml dependencies...]"
if grep -q "winres" Cargo.toml; then
    echo "   ✓ winres dependency found"
else
    echo "   ✗ winres dependency NOT FOUND"
fi

echo
echo "[4. Checking macOS .app build scripts...]"
if [ -f "build-macos-app.sh" ]; then
    echo "   ✓ build-macos-app.sh exists"
    if grep -q "icon.icns" build-macos-app.sh; then
        echo "     ✓ References icon.icns"
    else
        echo "     ✗ Does NOT reference icon.icns"
    fi
else
    echo "   ✗ build-macos-app.sh NOT FOUND"
fi

if [ -f "build-macos-app-wrapper.sh" ]; then
    echo "   ✓ build-macos-app-wrapper.sh exists"
    if grep -q "icon.icns" build-macos-app-wrapper.sh; then
        echo "     ✓ References icon.icns"
    else
        echo "     ✗ Does NOT reference icon.icns"
    fi
else
    echo "   ✗ build-macos-app-wrapper.sh NOT FOUND"
fi

echo
echo "[5. Checking compiled executable (if exists)...]"
if [ -f "target/release/openuo-launcher" ]; then
    SIZE=$(ls -lh target/release/openuo-launcher | awk '{print $5}')
    echo "   ✓ target/release/openuo-launcher exists (Size: $SIZE)"
else
    echo "   ⚠ target/release/openuo-launcher not built yet"
    echo "     Run 'cargo build --release' to build"
fi

echo
echo "[Summary]"
echo "========================================"
echo "Icon configuration check complete!"
echo
echo "Next steps:"
echo "1. If any files are missing, create them"
echo "2. Run 'cargo build --release' to build"
echo "3. For macOS .app: Run './build-macos-app-wrapper.sh'"
echo "4. For Windows installer: Run 'build-windows-installer.bat' on Windows"
echo
