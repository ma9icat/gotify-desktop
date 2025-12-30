# Gotify Desktop Client

<div align="center">

![许可证](https://img.shields.io/badge/License-MIT-blue.svg)
![Rust](https://img.shields.io/badge/Rust-1.70+-orange.svg)
![Tauri](https://img.shields.io/badge/Tauri-2.x-purple.svg)
![平台](https://img.shields.io/badge/Platform-Windows%20%7C%20macOS%20%7C%20Linux-green.svg)

基于 Rust 和 Tauri 构建的现代化跨平台 [Gotify](https://gotify.net/) 桌面客户端。

[English](README.md) | [中文](README_CN.md)

</div>

---

## ✨ 功能特性

- 🔌 **连接管理** - 安全存储 Gotify 服务器连接
- 📬 **消息管理** - 查看、刷新、删除消息
- 🎨 **优先级显示** - 按优先级（0-5）颜色区分消息
- 🔄 **自动刷新** - 30 秒自动刷新消息列表
- 🌙 **现代化界面** - 响应式设计，清晰的信息层级
- 🔒 **本地运行** - 数据仅在本地处理，不上传到第三方

### 🚧 即将推出

- 📡 **WebSocket 实时消息** - 消息实时推送
- 🔔 **系统通知** - 新消息桌面通知
- 💾 **配置持久化** - 自动记住服务器配置
- 🌓 **暗色模式** - 护眼主题切换

---

## 📦 安装

### 前置要求

- **Rust** 1.70+ - [安装指南](https://rustup.rs/)
- **Node.js** 18+ - [下载页面](https://nodejs.org/)
- **系统依赖**：
  - Linux: `libwebkit2gtk-4.0-dev libappindicator3-dev librsvg2-dev`
  - Windows/macOS: Tauri 自动处理

### 安装步骤

```bash
# 1. 克隆项目
git clone https://github.com/yourusername/gotify-desktop.git
cd gotify-desktop

# 2. 安装依赖
npm install

# 3. 开发模式运行
npm run tauri dev

# 4. 构建生产版本
npm run tauri build
```

### 快速启动

开发模式（推荐）：
```bash
npm run tauri dev
```

这将启动一个带有热重载的开发窗口。

---

## 🏗️ 构建

### 构建命令

```bash
# Debug 构建
cd src-tauri && cargo build

# Release 构建（优化）
cd src-tauri && cargo build --release

# 仅构建前端
npm run build
```

### 平台支持

| 平台 | 输出格式 | 命令 |
|------|----------|------|
| Windows | `.msi` / `.exe` | `npm run tauri build` |
| macOS | `.dmg` / `.app` | `npm run tauri build` |
| Linux | `.deb` / `.rpm` / `.AppImage` | `npm run tauri build` |

---

## 🧪 测试

```bash
# 运行 Rust 单元测试
cd src-tauri && cargo test

# 代码格式检查
rustfmt --check src-tauri/src/*.rs

# Clippy 静态分析
cargo clippy --all-features
```

---

## 📁 项目结构

```
gotify-desktop/
├── src/                          # 前端资源
│   ├── index.html               # 主页面 + 样式
│   └── main.js                  # 前端逻辑
├── src-tauri/                   # Tauri/Rust 后端
│   ├── src/
│   │   ├── main.rs              # 应用入口 + Tauri 命令
│   │   ├── gotify.rs            # Gotify API 客户端
│   │   └── tests.rs             # 单元测试
│   ├── Cargo.toml               # Rust 配置
│   └── tauri.conf.json          # Tauri 配置
├── .github/workflows/           # CI/CD
│   └── ci.yml                   # GitHub Actions
├── package.json                 # NPM 脚本
└── README.md                    # 项目说明（英文）
```

---

## 🛠️ 开发

### 技术栈

| 组件 | 技术 | 版本 |
|------|------|------|
| 框架 | Tauri | 2.x |
| 后端语言 | Rust | 1.70+ |
| 前端语言 | JavaScript | ES6+ |
| HTTP 客户端 | reqwest | 0.12 |
| 异步运行时 | Tokio | 1.x |
| 包管理 | npm | - |

### 开发命令

```bash
# 启动开发服务器（热重载）
npm run tauri dev

# 运行测试
cd src-tauri && cargo test

# 运行 lint
cargo clippy

# 代码格式化
cargo fmt
```

---

## 📝 API 参考

### Tauri 命令

| 命令 | 描述 |
|------|------|
| `connect_to_gotify` | 连接到 Gotify 服务器 |
| `fetch_messages` | 获取消息列表 |
| `delete_message` | 删除消息 |
| `disconnect_gotify` | 断开连接 |

### 消息结构

```json
{
  "id": 1,
  "message": "通知内容",
  "title": "标题（可选）",
  "priority": 3,
  "timestamp": "2024-01-01T00:00:00Z",
  "appid": 1,
  "extras": {}
}
```

---

## 🤝 贡献

欢迎贡献代码！请先阅读 CONTRIBUTING.md。

1. Fork 本项目
2. 创建分支 (`git checkout -b feature/amazing-feature`)
3. 提交更改 (`git commit -m 'Add amazing feature'`)
4. 推送到分支 (`git push origin feature/amazing-feature`)
5. 发起 Pull Request

---

## 📄 许可证

本项目基于 MIT 许可证开源 - 查看 [LICENSE](LICENSE) 文件了解详情。

---

## 🙏 致谢

- [Gotify](https://gotify.net/) - 简单的消息推送服务
- [Tauri](https://tauri.app/) - 轻量级桌面应用框架
- [Rust](https://www.rust-lang.org/) - 系统级编程语言

---

<div align="center">

**如果这个项目对你有帮助，请给一个 ⭐ Star！**

</div>