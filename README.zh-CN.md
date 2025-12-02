# OpenUO Launcher

[English](README.md) | 简体中文

现代化的 OpenUO 启动器，使用 Rust 编写，支持 HiDPI 显示器和自动更新。

[OpenUO](https://github.com/openuo-online/OpenUO) 是Fork了TazUO代码后，加入了汉化，以及一些使用体验的修改后的UO客户端，与该项目配套使用更加丝滑。

<img width="1072" height="744" alt="image" src="https://github.com/user-attachments/assets/a8133599-4faa-43c2-b8df-d696a9ef7cc0" />

## ✨ 特性

- 🎨 现代化 UI，支持 Retina/HiDPI 显示器
- 🔄 一键自动更新 OpenUO 和 Launcher
- 📦 多配置管理，轻松切换服务器
- 🔐 密码加密保存
- 🌍 跨平台支持（Windows、macOS Intel/ARM、Linux x64）
- 🌐 多语言支持（中文、English）

## 📥 下载

访问 [Releases](https://github.com/openuo-online/OpenUO-Launcher/releases/latest) 下载最新版本

## 🚀 快速开始

1. 下载并运行 Launcher
2. 点击"下载 OpenUO"自动安装客户端
3. 配置服务器和账号
4. 启动游戏

## 🛠️ 开发

```bash
# 克隆项目
git clone https://github.com/openuo-online/OpenUO-Launcher.git
cd OpenUO-Launcher

# 运行
cargo run

# 构建
./build.sh        # macOS/Linux (自动检测平台)
build.bat         # Windows
```

### Linux 依赖

在 Linux 上构建需要安装以下依赖：

```bash
# Ubuntu/Debian
sudo apt-get install libgtk-3-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev libssl-dev

# Fedora/RHEL
sudo dnf install gtk3-devel libxcb-devel libxkbcommon-devel openssl-devel

# Arch Linux
sudo pacman -S gtk3 libxcb libxkbcommon openssl
```

## 📝 配置文件

配置存储在 `Profiles/` 目录：
- `{uuid}.json` - 档案索引（名称、角色等）
- `Settings/{uuid}.json` - 详细设置（服务器、账号等）

## 🌐 自定义更新源

如果遇到 GitHub API 速率限制（403 错误），可以配置自己的 CDN：

在 Launcher 同目录创建 `update_source.json`：

```json
{
  "openuo_url": "https://your-cdn.com/openuo/latest.json",
  "launcher_url": "https://your-cdn.com/launcher/latest.json",
  "use_github_format": false
}
```

详细配置方法请参考：[自定义更新源文档](docs/CUSTOM_UPDATE_SOURCE.zh-CN.md)

## 🎯 HiDPI 支持

自动检测屏幕分辨率和缩放因子，传递给 OpenUO：
- `launcher_screen_width/height` - 屏幕尺寸
- `launcher_scale_factor` - 缩放因子（Retina 为 2.0）
- `launcher_is_hidpi` - 是否为高分辨率屏幕

## 🗺️ 路线图

### 计划中的功能

- [ ] **Manifest 客户端管理** - 基于 manifest 文件检测和更新私有客户端补丁
- [ ] **私钥加密通信** - 使用私有密钥加密客户端与服务器通信
- [ ] **WebSocket 网页端** - 配合 UO 网关和代理，支持浏览器直接游玩
- [ ] **独立助手窗体** - 类似 Orion UO 的助手功能，独立窗口管理

### 欢迎贡献

如果你对这些功能感兴趣或有其他想法，欢迎提交 [Issue](https://github.com/openuo-online/OpenUO-Launcher/issues) 讨论！

## 📄 许可证

GPL-3.0 - 详见 [LICENSE](LICENSE)

## 🙏 致谢

- [TazUO](https://github.com/PlayTazUO/TazUO) - TazUO
- [egui](https://github.com/emilk/egui) - UI 框架
