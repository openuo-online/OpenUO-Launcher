# 图标设置说明

## 图标类型说明

本项目支持三种图标：

### 1. 窗口图标（Window Icon）✅ 已配置
- **用途**: 窗口标题栏、任务栏/Dock 图标
- **平台**: Windows、macOS、Linux
- **文件**: `assets/centerlogo.png`
- **状态**: ✅ 已自动加载，无需额外配置

### 2. Windows 可执行文件图标（.exe Icon）⚠️ 需要配置
- **用途**: 文件资源管理器中的 .exe 文件图标
- **平台**: 仅 Windows
- **文件**: `assets/icon.ico`（需要创建）
- **状态**: ⚠️ 需要添加 .ico 文件

### 3. macOS 应用程序图标（.app Bundle Icon）
- **用途**: Finder 和 Dock 中的应用图标
- **平台**: 仅 macOS
- **文件**: 需要 .icns 格式和 Info.plist 配置
- **状态**: 需要额外配置（打包成 .app 时）

---

## 如何添加 Windows .exe 文件图标

### 步骤 1: 准备图标文件

**选项 A: 在线转换（推荐）**
1. 访问 https://convertio.co/png-ico/
2. 上传 `assets/centerlogo.png`
3. 下载转换后的 `icon.ico`
4. 放到 `assets/` 目录

**选项 B: 使用 ImageMagick**
```bash
magick convert assets/centerlogo.png -define icon:auto-resize=256,128,64,48,32,16 assets/icon.ico
```

**选项 C: 使用在线工具**
- https://www.icoconverter.com/
- https://favicon.io/favicon-converter/

### 步骤 2: 重新编译
```bash
cargo build --release
```
或
```bash
build.bat
```

---

## 如何配置 macOS .app 图标

### 步骤 1: 创建 .icns 文件

```bash
# 创建临时目录
mkdir icon.iconset

# 生成不同尺寸
sips -z 16 16     centerlogo.png --out icon.iconset/icon_16x16.png
sips -z 32 32     centerlogo.png --out icon.iconset/icon_16x16@2x.png
sips -z 32 32     centerlogo.png --out icon.iconset/icon_32x32.png
sips -z 64 64     centerlogo.png --out icon.iconset/icon_32x32@2x.png
sips -z 128 128   centerlogo.png --out icon.iconset/icon_128x128.png
sips -z 256 256   centerlogo.png --out icon.iconset/icon_128x128@2x.png
sips -z 256 256   centerlogo.png --out icon.iconset/icon_256x256.png
sips -z 512 512   centerlogo.png --out icon.iconset/icon_256x256@2x.png
sips -z 512 512   centerlogo.png --out icon.iconset/icon_512x512.png
sips -z 1024 1024 centerlogo.png --out icon.iconset/icon_512x512@2x.png

# 转换为 .icns
iconutil -c icns icon.iconset -o assets/icon.icns

# 清理
rm -rf icon.iconset
```

### 步骤 2: 打包成 .app（需要额外工具）

可以使用 `cargo-bundle` 或手动创建 .app bundle。

---

## 当前状态总结

| 图标类型 | 状态 | 文件 | 说明 |
|---------|------|------|------|
| 窗口图标 | ✅ 完成 | `centerlogo.png` | 运行时自动加载 |
| Windows .exe | ⚠️ 待配置 | `icon.ico` (需创建) | 需要添加 .ico 文件 |
| macOS .app | ⚠️ 待配置 | `icon.icns` (需创建) | 需要打包成 .app |

---

## 技术细节

### 窗口图标实现
- 使用 `winit::window::Icon::from_rgba()`
- 从嵌入的 PNG 图片加载
- 跨平台支持（Windows、macOS、Linux）
- 代码位置: `src/main.rs` 中的 `load_window_icon()` 函数

### Windows .exe 图标实现
- 使用 `winres` crate
- 在编译时嵌入资源
- 代码位置: `build.rs`

### macOS .app 图标实现
- 需要 .icns 格式
- 通过 Info.plist 配置
- 需要打包工具（如 cargo-bundle）
