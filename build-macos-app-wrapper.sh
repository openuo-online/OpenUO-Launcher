#!/bin/bash
set -e

echo "[Building OpenUO Launcher for macOS (with auto-update support)...]"
echo

# 获取版本号
VERSION=$(grep '^version' Cargo.toml | head -1 | cut -d'"' -f2)
echo "[Version: $VERSION]"

# 检测架构
CURRENT_ARCH=$(uname -m)
if [[ "$CURRENT_ARCH" == "arm64" ]]; then
    ARCH_NAME="arm64"
else
    ARCH_NAME="x64"
fi

echo "[Architecture: $ARCH_NAME]"
echo

# 构建 release 版本
echo "[Building release binary...]"
cargo build --release

if [ $? -ne 0 ]; then
    echo "[Build failed!]"
    exit 1
fi

# 创建 .app 目录结构
APP_NAME="OpenUO Launcher.app"
APP_DIR="releases/$APP_NAME"
CONTENTS_DIR="$APP_DIR/Contents"
MACOS_DIR="$CONTENTS_DIR/MacOS"
RESOURCES_DIR="$CONTENTS_DIR/Resources"

echo "[Creating .app bundle structure...]"
rm -rf "$APP_DIR"
mkdir -p "$MACOS_DIR"
mkdir -p "$RESOURCES_DIR"

# 创建启动脚本（wrapper）
# 这个脚本会把真正的可执行文件复制到用户目录，然后运行它
# 这样自动更新时只需要替换用户目录的文件，不会破坏 .app 的签名
echo "[Creating launcher wrapper...]"
cat > "$MACOS_DIR/OpenUO Launcher" << 'WRAPPER_EOF'
#!/bin/bash

# 用户数据目录
APP_SUPPORT="$HOME/Library/Application Support/OpenUO Launcher"
BINARY_DIR="$APP_SUPPORT/bin"
BINARY_PATH="$BINARY_DIR/openuo-launcher"

# 获取 .app 内嵌入的二进制文件路径
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
EMBEDDED_BINARY="$SCRIPT_DIR/openuo-launcher-embedded"

# 确保目录存在
mkdir -p "$BINARY_DIR"

# 如果用户目录没有可执行文件，或者嵌入的版本更新，则复制
if [ ! -f "$BINARY_PATH" ] || [ "$EMBEDDED_BINARY" -nt "$BINARY_PATH" ]; then
    echo "Installing/updating launcher binary..."
    cp "$EMBEDDED_BINARY" "$BINARY_PATH"
    chmod +x "$BINARY_PATH"
fi

# 运行真正的可执行文件
exec "$BINARY_PATH" "$@"
WRAPPER_EOF

chmod +x "$MACOS_DIR/OpenUO Launcher"

# 复制真正的可执行文件（作为嵌入的二进制）
echo "[Copying embedded binary...]"
cp target/release/openuo-launcher "$MACOS_DIR/openuo-launcher-embedded"
chmod +x "$MACOS_DIR/openuo-launcher-embedded"

# 复制图标（如果存在）
if [ -f "assets/icon.icns" ]; then
    echo "[Copying icon...]"
    cp assets/icon.icns "$RESOURCES_DIR/AppIcon.icns"
fi

# 创建 Info.plist
echo "[Creating Info.plist...]"
cat > "$CONTENTS_DIR/Info.plist" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleDevelopmentRegion</key>
    <string>en</string>
    <key>CFBundleExecutable</key>
    <string>OpenUO Launcher</string>
    <key>CFBundleIconFile</key>
    <string>AppIcon</string>
    <key>CFBundleIdentifier</key>
    <string>online.openuo.launcher</string>
    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>
    <key>CFBundleName</key>
    <string>OpenUO Launcher</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>$VERSION</string>
    <key>CFBundleVersion</key>
    <string>$VERSION</string>
    <key>LSMinimumSystemVersion</key>
    <string>10.13</string>
    <key>NSHighResolutionCapable</key>
    <true/>
    <key>NSSupportsAutomaticGraphicsSwitching</key>
    <true/>
</dict>
</plist>
EOF

echo
echo "[✓] Build complete!"
echo "[Location: $APP_DIR]"
echo

# 显示大小
SIZE=$(du -sh "$APP_DIR" | cut -f1)
echo "[Size: $SIZE]"
echo
echo "[Note: Auto-update will work correctly with this .app bundle]"
echo "[The actual binary will be stored in: ~/Library/Application Support/OpenUO Launcher/bin/]"

# 创建 DMG（可选）
read -p "[Create DMG? (y/n)] " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    DMG_NAME="releases/OpenUO-Launcher-macos-$ARCH_NAME-v$VERSION.dmg"
    echo "[Creating DMG...]"
    
    # 删除旧的 DMG
    rm -f "$DMG_NAME"
    
    # 创建临时目录
    TMP_DIR=$(mktemp -d)
    cp -R "$APP_DIR" "$TMP_DIR/"
    
    # 创建 DMG
    hdiutil create -volname "OpenUO Launcher" -srcfolder "$TMP_DIR" -ov -format UDZO "$DMG_NAME"
    
    # 清理
    rm -rf "$TMP_DIR"
    
    echo "[✓] DMG created: $DMG_NAME]"
fi

echo
echo "[Done!]"
