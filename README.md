# OpenUO Launcher

English | [简体中文](README.zh-CN.md)

A modern OpenUO launcher written in Rust, with HiDPI display support and automatic updates.

[OpenUO](https://github.com/openuo-online/OpenUO) is a fork of TazUO with Chinese localization and enhanced user experience improvements, designed to work seamlessly with this launcher.

<img width="1072" height="744" alt="image" src="https://github.com/user-attachments/assets/3bdcfc46-ceb7-4b53-bac1-be1387db0f6f" />


## ✨ Features

- 🎨 Modern UI with Retina/HiDPI display support
- 🔄 One-click automatic updates for OpenUO and Launcher
- 📦 Multiple profile management for easy server switching
- 🔐 Encrypted password storage
- 🌍 Cross-platform support (Windows, macOS Intel/ARM, Linux x64)
- 🌐 Multi-language support (Chinese, English)

## 📥 Download

Visit [Releases](https://github.com/openuo-online/OpenUO-Launcher/releases/latest) to download the latest version

## 🚀 Quick Start

1. Download and run the Launcher
2. Click "Download OpenUO" to automatically install the client
3. Configure server and account settings
4. Launch the game

## 🛠️ Development

```bash
# Clone the repository
git clone https://github.com/openuo-online/OpenUO-Launcher.git
cd OpenUO-Launcher

# Run
cargo run

# Build
./build.sh        # macOS/Linux (auto-detect platform)
build.bat         # Windows
```

### Linux Dependencies

Building on Linux requires the following dependencies:

```bash
# Ubuntu/Debian
sudo apt-get install libgtk-3-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev libssl-dev

# Fedora/RHEL
sudo dnf install gtk3-devel libxcb-devel libxkbcommon-devel openssl-devel

# Arch Linux
sudo pacman -S gtk3 libxcb libxkbcommon openssl
```

## 📝 Configuration Files

Configurations are stored in the `Profiles/` directory:
- `{uuid}.json` - Profile index (name, character, etc.)
- `Settings/{uuid}.json` - Detailed settings (server, account, etc.)

## 🌐 Custom Update Source

If you encounter GitHub API rate limits (403 errors), you can configure your own CDN:

Create `update_source.json` in the same directory as the Launcher:

```json
{
  "openuo_url": "https://your-cdn.com/openuo/latest.json",
  "launcher_url": "https://your-cdn.com/launcher/latest.json",
  "use_github_format": false
}
```

For detailed configuration, see: [Custom Update Source Documentation](docs/CUSTOM_UPDATE_SOURCE.md)

## 🎯 HiDPI Support

Automatically detects screen resolution and scaling factor, passed to OpenUO:
- `launcher_screen_width/height` - Screen dimensions
- `launcher_scale_factor` - Scaling factor (2.0 for Retina)
- `launcher_is_hidpi` - Whether it's a high-resolution display

## 🗺️ Roadmap

### Planned Features

- [ ] **Manifest Client Management** - Detect and update private client patches based on manifest files
- [ ] **Private Key Encrypted Communication** - Encrypt client-server communication using private keys
- [ ] **WebSocket Web Client** - Support browser-based gameplay with UO gateway and proxy
- [ ] **Standalone Assistant Window** - Independent window management similar to Orion UO assistant features

### Contributions Welcome

If you're interested in these features or have other ideas, feel free to submit an [Issue](https://github.com/openuo-online/OpenUO-Launcher/issues) for discussion!

## 📄 License

GPL-3.0 - See [LICENSE](LICENSE) for details

## 🙏 Acknowledgments

- [TazUO](https://github.com/PlayTazUO/TazUO) - TazUO
- [egui](https://github.com/emilk/egui) - UI Framework
