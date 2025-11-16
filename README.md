# Another OpenUO Launcher

现代化的 OpenUO 启动器，使用 Rust 编写，支持 HiDPI 显示器和自动更新。

[OpenUO](https://github.com/openuo-online/OpenUO) 是Fork了TazUO代码后，加入了汉化，以及一些使用体验的修改后的客户端软件，与该项目配套使用更加丝滑。

<img width="1072" height="744" alt="image" src="https://github.com/user-attachments/assets/a8133599-4faa-43c2-b8df-d696a9ef7cc0" />

## ✨ 特性

- 🎨 现代化 UI，支持 Retina/HiDPI 显示器
- 🔄 一键自动更新 OpenUO 和 Launcher
- 📦 多配置管理，轻松切换服务器
- 🔐 密码加密保存
- 🌍 跨平台支持（Windows、macOS Intel/ARM）

## 📥 下载

访问 [Releases](https://github.com/openuo-online/Another-OpenUO-Launcher/releases/latest) 下载最新版本

## 🚀 快速开始

1. 下载并运行 Launcher
2. 点击"下载 OpenUO"自动安装客户端
3. 配置服务器和账号
4. 启动游戏

## 🛠️ 开发

```bash
# 克隆项目
git clone https://github.com/openuo-online/Another-OpenUO-Launcher.git
cd Another-OpenUO-Launcher

# 运行
cargo run

# 构建
./build.sh        # macOS/Linux
build.bat         # Windows
```

## 📝 配置文件

配置存储在 `Profiles/` 目录：
- `{uuid}.json` - 档案索引（名称、角色等）
- `Settings/{uuid}.json` - 详细设置（服务器、账号等）

## 🎯 HiDPI 支持

自动检测屏幕分辨率和缩放因子，传递给 OpenUO：
- `launcher_screen_width/height` - 屏幕尺寸
- `launcher_scale_factor` - 缩放因子（Retina 为 2.0）
- `launcher_is_hidpi` - 是否为高分辨率屏幕

## 📄 许可证

GPL-3.0 - 详见 [LICENSE](LICENSE)

## 🙏 致谢

- [TazUO](https://github.com/PlayTazUO/TazUO) - TazUO
- [egui](https://github.com/emilk/egui) - UI 框架
