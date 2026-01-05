# Gotify Desktop Client

<div align="center">

![License](https://img.shields.io/badge/License-MIT-blue.svg)
![Rust](https://img.shields.io/badge/Rust-1.70+-orange.svg)
![Tauri](https://img.shields.io/badge/Tauri-2.x-purple.svg)
![Platform](https://img.shields.io/badge/Platform-Windows%20%7C%20macOS%20%7C%20Linux-green.svg)

A modern, cross-platform desktop client for [Gotify](https://gotify.net/) built with Rust and Tauri.

[English](README.md) | [中文](README_CN.md)

</div>

---

## ✨ Features

- 🔌 **Connection Management** - Secure storage of Gotify server connections with multi-server support
- 📬 **Message Management** - View, refresh, delete messages with pagination support
- 🎨 **Priority Display** - Color-coded messages by priority (0-5)
- 📡 **WebSocket Real-time Messages** - Real-time message push without manual refresh
- 🔔 **System Notifications** - Desktop notifications for new messages
- 💾 **Configuration Persistence** - Auto-save server configurations and app settings
- 🎯 **System Tray** - Minimize to system tray with tray menu support
- ⚙️ **App Settings** - Autostart, silent start, tray run, notification toggle
- 🌙 **Modern UI** - Responsive design with clear information hierarchy and collapsible sidebar
- 🔒 **Local Execution** - Data processed locally, no third-party uploads

### 🚧 Coming Soon

- 🌓 **Dark Mode** - Eye-friendly theme toggle
- 🔍 **Message Search** - Search messages by keywords
- 📥 **Message Export** - Export to JSON/CSV formats

### 🆕 v0.4.0 Features

- 🔄 **Auto Update** - Built-in update system with one-click installation
- 🔒 **Security** - Disabled developer tools in production builds
- 🎨 **UI Improvements** - Added sidebar icons using Remix Icon
- 🐛 **Bug Fixes** - Fixed duplicate messages and pagination issues

---

## 📦 Installation

### Prerequisites

- **Rust** 1.70+ - [Installation Guide](https://rustup.rs/)
- **Node.js** 18+ - [Download Page](https://nodejs.org/)
- **System Dependencies**:
  - Linux: `libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev`
  - Windows/macOS: Tauri handles automatically

### Installation Steps

```bash
# 1. Clone the repository
git clone https://github.com/ma9icat/gotify-desktop.git
cd gotify-desktop

# 2. Install dependencies
npm install

# 3. Run in development mode
npm run tauri dev

# 4. Build production version
npm run tauri build
```

### Quick Start

Development mode (recommended):
```bash
npm run tauri dev
```

This will launch a development window with hot reload.

---

## 🏗️ Building

### Build Commands

```bash
# Debug build
cd src-tauri && cargo build

# Release build (optimized)
cd src-tauri && cargo build --release

# Frontend build only
npm run build
```

### Platform Support

| Platform | Output Format | Command |
|----------|---------------|---------|
| Windows | `.msi` / `.exe` | `npm run tauri build` |
| macOS | `.dmg` / `.app` | `npm run tauri build` |
| Linux | `.deb` / `.rpm` / `.AppImage` | `npm run tauri build` |

---

## 🧪 Testing

```bash
# Run Rust unit tests
cd src-tauri && cargo test

# Code format check
rustfmt --check src-tauri/src/*.rs

# Clippy static analysis
cargo clippy --all-features
```

---

## 📁 Project Structure

```
gotify-desktop/
├── src/                          # Frontend resources
│   ├── index.html               # Main page + styles
│   └── main.js                  # Frontend logic (state management, event handling)
├── src-tauri/                   # Tauri/Rust backend
│   ├── src/
│   │   ├── main.rs              # App entry point + Tauri commands
│   │   ├── gotify.rs            # Gotify API client
│   │   └── tests.rs             # Unit tests
│   ├── Cargo.toml               # Rust configuration
│   ├── tauri.conf.json          # Tauri configuration
│   ├── build.rs                 # Build script
│   ├── capabilities/            # Tauri 2.x permission configuration
│   │   └── default.json
│   └── icons/                   # App icons
│       ├── icon.ico
│       └── icon.png
├── .github/workflows/           # CI/CD
│   └── ci.yml                   # GitHub Actions
├── package.json                 # NPM scripts
└── README.md                    # Project documentation
```

---

## 🛠️ Development

### Tech Stack

| Component | Technology | Version |
|-----------|------------|---------|
| Framework | Tauri | 2.x |
| Backend Language | Rust | 1.70+ |
| Frontend Language | JavaScript | ES6+ |
| HTTP Client | reqwest | 0.12 |
| Async Runtime | Tokio | 1.x |
| Package Manager | npm | - |

### Development Commands

```bash
# Start dev server (hot reload)
npm run tauri dev

# Run tests
cd src-tauri && cargo test

# Run lint
cargo clippy

# Code formatting
cargo fmt
```

---

## 📝 API Reference

### Tauri Commands

| Command | Description |
|---------|-------------|
| `connect_to_gotify` | Connect to Gotify server (with WebSocket support) |
| `fetch_messages` | Fetch message list (with pagination and incremental fetch) |
| `delete_message` | Delete message |
| `disconnect_gotify` | Disconnect from server |
| `get_health` | Health check |
| `create_message` | Create message |
| `get_applications` | Get application list |
| `save_config` | Save server configuration |
| `get_configs` | Get configuration list |
| `delete_config` | Delete configuration |
| `update_config` | Update configuration |
| `set_default_config` | Set default configuration |
| `get_default_config` | Get default configuration |
| `get_app_settings` | Get app settings |
| `update_app_settings` | Update app settings |
| `toggle_autostart` | Toggle autostart |
| `show_window` | Show window |
| `hide_window` | Hide window |
| `send_notification` | Send system notification |

### Message Structure

```json
{
  "id": 1,
  "message": "Notification content",
  "title": "Title (optional)",
  "priority": 3,
  "timestamp": "2024-01-01T00:00:00Z",
  "app_id": 1,
  "extras": {}
}
```

---

## 🤝 Contributing

Contributions are welcome! Please read [CONTRIBUTING.md](CONTRIBUTING.md) first.

1. Fork this repository
2. Create a branch (`git checkout -b feature/amazing-feature`)
3. Commit changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

---

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

## 🙏 Acknowledgments

- [Gotify](https://gotify.net/) - Simple message push service
- [Tauri](https://tauri.app/) - Lightweight desktop application framework
- [Rust](https://www.rust-lang.org/) - Systems programming language

**Development Tool:**

This project is developed and maintained entirely using [iFlow CLI](https://iflow.dev). iFlow CLI is an intelligent code assistance tool that helps efficiently complete code analysis, implementation, and documentation tasks.

---

<div align="center">

**If this project helps you, please give it a ⭐ Star!**

</div>