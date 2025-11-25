#!/bin/bash
set -e

echo "[Building OpenUO Launcher for macOS...]"
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

# 复制可执行文件
echo "[Copying executable...]"
cp target/release/openuo-launcher "$MACOS_DIR/OpenUO Launcher"
chmod +x "$MACOS_DIR/OpenUO Launcher"

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
