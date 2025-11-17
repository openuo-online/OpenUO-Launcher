#!/bin/bash
set -e

echo "[Building OpenUO Launcher for Linux...]"
echo

# 读取版本号
VERSION=$(grep '^version' Cargo.toml | head -1 | cut -d'"' -f2)
echo "[Version: $VERSION]"
echo

# 创建输出目录
mkdir -p releases

# 检测平台
echo "[Platform: Linux x64]"
echo

# 构建
echo "[Building release...]"
cargo build --release

if [ $? -ne 0 ]; then
    echo "[Build failed!]"
    exit 1
fi

# 复制到 releases 目录
OUTPUT="releases/OpenUO-Launcher-linux-x64-v${VERSION}"
cp target/release/openuo-launcher "$OUTPUT"

# 添加执行权限
chmod +x "$OUTPUT"

echo
echo "[Build complete: $OUTPUT]"
echo

# 显示文件大小
SIZE=$(stat -f%z "$OUTPUT" 2>/dev/null || stat -c%s "$OUTPUT" 2>/dev/null)
SIZE_MB=$((SIZE / 1048576))
echo "[File size: ${SIZE_MB} MB]"

echo
echo "[Done!]"
