# Windows 安装程序打包指南

本项目提供两种 Windows 安装程序打包方案：

## 方案 1: Inno Setup（推荐）

### 优点
- 简单易用，配置直观
- 生成的 EXE 安装程序体积小
- 支持多语言界面
- 自动检测并卸载旧版本

### 安装步骤

1. **下载并安装 Inno Setup**
   - 访问：https://jrsoftware.org/isdl.php
   - 下载并安装最新版本（推荐 6.x）

2. **构建安装程序**
   ```batch
   build-windows-installer.bat
   ```

3. **输出文件**
   - 位置：`releases/OpenUO-Launcher-Setup-vX.X.X.exe`
   - 这是一个完整的安装程序，包含卸载功能

### 自定义配置

编辑 `installer.iss` 文件可以修改：
- 安装目录
- 快捷方式位置
- 安装界面语言
- 文件关联等

---

## 方案 2: WiX Toolset（专业）

### 优点
- 生成标准的 MSI 安装包
- 企业环境友好（支持 GPO 部署）
- 更好的升级管理
- 符合 Windows Installer 标准

### 安装步骤

1. **下载并安装 WiX Toolset**
   - 访问：https://wixtoolset.org/releases/
   - 下载并安装 WiX Toolset v3.11 或更高版本

2. **构建 MSI 安装包**
   ```batch
   build-windows-msi.bat
   ```

3. **输出文件**
   - 位置：`releases/OpenUO-Launcher-vX.X.X.msi`
   - 标准 Windows Installer 包

### 自定义配置

编辑 `installer.wxs` 文件可以修改：
- 安装目录结构
- 组件和功能
- 注册表项
- 服务安装等

---

## 自动更新兼容性

两种安装方式都与现有的自动更新机制兼容：

### Inno Setup
- 默认安装到 `%LOCALAPPDATA%\OpenUO Launcher`
- 用户权限即可更新，无需管理员权限

### WiX MSI
- 配置为 `perUser` 安装
- 自动更新时可以直接替换可执行文件

### 注意事项

如果安装到 `Program Files`，自动更新可能需要管理员权限。建议：
1. 安装到用户目录（默认配置）
2. 或者使用类似 macOS 的 wrapper 方案

---

## 对比总结

| 特性 | Inno Setup | WiX Toolset |
|------|-----------|-------------|
| 易用性 | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ |
| 安装包格式 | EXE | MSI |
| 文件大小 | 较小 | 较大 |
| 企业部署 | 支持 | ⭐⭐⭐⭐⭐ |
| 自定义界面 | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ |
| 学习曲线 | 平缓 | 陡峭 |

**推荐选择**：
- 个人用户/小团队：使用 **Inno Setup**
- 企业环境/需要 MSI：使用 **WiX Toolset**

---

## 常见问题

### Q: 安装程序需要签名吗？
A: 建议签名以避免 Windows SmartScreen 警告。需要代码签名证书（约 $100-300/年）。

### Q: 如何添加更多文件到安装包？
A: 
- **Inno Setup**: 在 `[Files]` 部分添加 `Source` 行
- **WiX**: 在 `<ComponentGroup>` 中添加 `<Component>` 和 `<File>`

### Q: 自动更新会破坏安装吗？
A: 不会。当前配置安装到用户目录，自动更新可以正常工作。

### Q: 可以静默安装吗？
A: 可以。
- **Inno Setup**: `OpenUO-Launcher-Setup.exe /SILENT`
- **WiX MSI**: `msiexec /i OpenUO-Launcher.msi /quiet`
