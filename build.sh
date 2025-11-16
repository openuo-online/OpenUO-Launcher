#!/bin/bash
set -e

echo "🚀 Building OpenUO Launcher..."
echo ""

VERSION=$(grep '^version' Cargo.toml | head -1 | cut -d'"' -f2)
echo "📦 Version: $VERSION"
echo ""

# 创建输出目录
mkdir -p releases

# 检测当前平台
CURRENT_OS=$(uname -s)
CURRENT_ARCH=$(uname -m)

echo "🖥️  Platform: $CURRENT_OS $CURRENT_ARCH"
echo ""

# 构建
echo "⚙️  Building release..."
cargo build --release

# 复制到 releases 目录
if [[ "$CURRENT_OS" == "Darwin" ]]; then
    if [[ "$CURRENT_ARCH" == "arm64" ]]; then
        OUTPUT="releases/OpenUO-Launcher-macos-arm64-v$VERSION"
    else
        OUTPUT="releases/OpenUO-Launcher-macos-x64-v$VERSION"
    fi
    cp target/release/openuo-launcher "$OUTPUT"
elif [[ "$CURRENT_OS" == "MINGW"* ]] || [[ "$CURRENT_OS" == "MSYS"* ]] || [[ "$CURRENT_OS" == "CYGWIN"* ]]; then
    OUTPUT="releases/OpenUO-Launcher-windows-x64-v$VERSION.exe"
    cp target/release/openuo-launcher.exe "$OUTPUT"
else
    OUTPUT="releases/OpenUO-Launcher-linux-v$VERSION"
    cp target/release/openuo-launcher "$OUTPUT"
fi

echo "✅ Build complete: $OUTPUT"
echo ""
echo "📦 File size: $(ls -lh "$OUTPUT" | awk '{print $5}')"
echo ""
echo "✨ Done!"
