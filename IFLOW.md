# Gotify Desktop 项目上下文文档

## 项目概述

Gotify Desktop 是一个跨平台的 Gotify 服务器桌面客户端应用，使用 Rust + Tauri 框架开发。该项目旨在为用户提供一个原生桌面应用来连接和交互 Gotify 消息服务器，支持消息获取、显示、删除等功能。

**技术栈：**
- **前端**：原生 HTML5/CSS3 + JavaScript (ES6+)（src/ 目录）
- **后端/框架**：Tauri 2.x（Rust） + Rust crates（src-tauri/ 目录）
- **通信**：HTTP REST API + WebSocket（预留）
- **异步运行时**：Tokio 1.x
- **HTTP 客户端**：reqwest 0.12

---

## 构建与运行

### 环境要求

1. **Rust** - 通过 [rustup.rs](https://rustup.rs/) 安装（推荐 rust-version: 1.70+）
2. **Node.js** - 推荐 LTS 版本 (20.x)
3. **系统依赖**：
   - Linux：`libwebkit2gtk-4.0-dev`, `libappindicator3-dev`, `librsvg2-dev`
   - Windows/macOS：Tauri 自动处理

### 常用命令

```bash
# 安装依赖
npm install

# 开发模式运行（热重载）
npm run tauri dev

# 构建生产版本（带优化）
npm run tauri build

# 运行 Rust 测试
cd src-tauri && cargo test

# 代码格式检查
rustfmt --check src-tauri/src/*.rs

# Clippy 静态分析
cargo clippy --all-features
```

### 项目结构

```
gotify-desktop/
├── src/                          # 前端资源目录
│   ├── index.html               # 主页面 HTML（含现代化 UI 样式）
│   └── main.js                  # 前端逻辑（含状态管理）
├── src-tauri/                   # Tauri/Rust 后端
│   ├── src/
│   │   ├── main.rs              # 应用入口点，定义 Tauri 命令
│   │   ├── gotify.rs            # Gotify API 客户端实现
│   │   └── tests.rs             # Rust 单元测试
│   ├── Cargo.toml               # Rust 依赖配置（已优化）
│   ├── tauri.conf.json          # Tauri 配置（窗口、打包等）
│   └── build.rs                 # 构建脚本
├── .github/workflows/           # CI/CD 配置
│   └── ci.yml                   # GitHub Actions 工作流
├── package.json                 # Node.js 依赖和脚本
└── README.md                    # 项目说明
```

---

## 核心功能

### 已实现功能

1. **服务器连接** (`connect_to_gotify` 命令)
   - 输入 Gotify 服务器 URL 和 Token
   - URL 格式验证
   - 建立持久化的客户端连接（存储在 `AppState` 中）
   - 统一 API 响应格式 (`ApiResponse<T>`)

2. **消息获取** (`fetch_messages` 命令)
   - 获取消息列表
   - 支持增量获取（`since` 参数用于获取新消息）
   - 30 秒自动刷新

3. **消息删除** (`delete_message` 命令)
   - 按 ID 删除单条消息
   - 删除后自动更新消息列表

4. **断开连接** (`disconnect_gotify` 命令)
   - 清除保存的客户端状态

5. **健康检查** (`get_health` 方法)
   - 检查 Gotify 服务器健康状态

### 消息优先级显示

消息按优先级（0-5）显示不同颜色：
- **优先级 0**：灰色 - 普通消息
- **优先级 1**：蓝色 - 低优先级
- **优先级 2**：黄色 - 中等优先级
- **优先级 3**：红色 - 高优先级
- **优先级 4-5**：深红色 - 紧急消息

---

## 代码架构

### 前端 (src/main.js)

**状态管理 (`AppState`):**
```javascript
const AppState = {
    connected: boolean,      // 连接状态
    serverUrl: string,       // 服务器地址
    messages: Message[],     // 消息列表
    loading: boolean,        // 加载状态
    error: string | null     // 错误信息
};
```

**核心函数：**
- `updateUIState(updates)` - 统一更新 UI 状态
- `render()` - 渲染整个界面
- `renderMessages()` - 渲染消息列表
- `refreshMessages()` - 刷新消息（带错误处理）
- `deleteMessage(id)` - 删除消息
- `escapeHtml()` - HTML 转义（防止 XSS）
- `formatTime()` - 时间格式化

**事件处理：**
- 连接按钮点击 → `connect_to_gotify`
- 断开连接按钮 → `disconnect_gotify`
- 刷新按钮 → `refreshMessages`
- 删除按钮 → `delete_message`
- 定时器 → 每 30 秒自动刷新

### 后端 (src-tauri/src/)

**main.rs:**
- `AppState` 封装：提供 `set_client()`, `clear_client()`, `get_client()` 方法
- `ApiResponse<T>` 统一响应格式
- Tauri 命令：`connect_to_gotify`, `fetch_messages`, `disconnect_gotify`, `delete_message`

**gotify.rs:**

**GotifyError 错误类型:**
```rust
pub enum GotifyError {
    InvalidUrl(url::ParseError),
    HttpError(reqwest::Error),
    JsonError(serde_json::Error),
    ServerError(String),      // 服务器返回错误
    AuthFailed,               // 认证失败
    NotFound(String),         // 资源不存在
    NotConnected,             // 未连接
    Unknown(String),          // 未知错误
}
```

**GotifyClient 方法:**
| 方法 | 说明 |
|------|------|
| `new(url, token)` | 创建客户端（带 30 秒超时） |
| `get_messages(since)` | 获取消息列表 |
| `delete_message(id)` | 删除消息 |
| `create_message(req)` | 创建消息 |
| `get_applications()` | 获取应用列表 |
| `get_health()` | 健康检查 |
| `handle_response()` | 统一 HTTP 响应处理 |

**tests.rs:**
- 消息/应用反序列化测试
- 错误类型 Display 测试
- URL 验证测试

---

## API 参考

### Rust 命令（frontend → backend）

| 命令 | 参数 | 返回 | 说明 |
|------|------|------|------|
| `connect_to_gotify` | `{ server_url: string, token: string }` | `ApiResponse<()>` | 连接到 Gotify 服务器 |
| `fetch_messages` | `since: Option<u64>` | `ApiResponse<Vec<Message>>` | 获取消息列表 |
| `disconnect_gotify` | 无 | `ApiResponse<()>` | 断开连接 |
| `delete_message` | `messageId: u64` | `ApiResponse<()>` | 删除消息 |

### 统一响应格式

```rust
struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    error: Option<String>,
}
```

### Rust 结构体

**Message**
```rust
pub struct Message {
    pub id: u64,
    pub message: String,
    pub title: Option<String>,
    pub priority: i32,
    pub timestamp: String,
    pub app_id: u64,
    pub extras: Option<serde_json::Value>,  // 新增：扩展字段
}
```

**Application**
```rust
pub struct Application {
    pub id: u64,
    pub name: String,
    pub description: Option<String>,  // 改为 Option
    pub token: Option<String>,
    pub internal: Option<bool>,       // 新增字段
}
```

**CreateMessageRequest**
```rust
pub struct CreateMessageRequest {
    pub message: String,
    pub title: Option<String>,
    pub priority: Option<i32>,
    pub extras: Option<serde_json::Value>,
}
```

---

## CI/CD 配置

**.github/workflows/ci.yml:**

**Jobs:**
1. **test** - 运行 Rust 测试、格式检查、Clippy 分析
2. **build** - 构建 Tauri 应用并上传 artifacts

**运行条件：**
- push 到 main/develop 分支
- pull request 合并到 main/develop

---

## 构建优化配置

**Cargo.toml Profile:**
```toml
[profile.release]
panic = "abort"
codegen-units = 1
lto = true           # 链接时优化
opt-level = "z"      # 优化大小
strip = true         # 去除符号表
```

**Tauri 打包配置:**
- 窗口大小：900x700（最小 600x400）
- 支持 Linux (.deb, .rpm, .AppImage)
- 支持 macOS (.dmg, .app)
- 支持 Windows (.msi, .exe)

---

## 开发约定

### 代码风格

- **Rust**：遵循 `rustfmt` 默认格式化
- **JavaScript**：ES6+ 语法，模块化组织
- **错误处理**：
  - Rust：使用 `thiserror` 定义错误类型，`ApiResponse<T>` 统一返回
  - 前端：try-catch + 状态管理 + UI 错误提示

### Tauri 配置

- 窗口大小：900x700（可调整）
- 权限：`dialog`, `fs`, `shell`, `window`
- CSP：未限制

### 状态管理

- **后端**：`AppState` (Mutex<Option<GotifyClient>>)
- **前端**：`AppState` 对象（集中管理所有 UI 状态）

### 测试

- **单元测试**：`src-tauri/src/tests.rs`
- 运行方式：`cd src-tauri && cargo test`

---

## 待实现功能

1. **WebSocket 实时消息**
   - 使用 `tokio-tungstenite` 实现
   - 前端集成事件监听

2. **配置持久化**
   - 使用 `tauri-plugin-store` 保存服务器配置
   - 记住最后连接的服务器

3. **系统通知**
   - 新消息到达时弹出系统通知

4. **暗色模式**
   - CSS 变量主题切换
   - 持久化主题偏好

5. **应用列表管理**
   - 前端调用 `get_applications` API
   - 显示和管理应用

---

## 版本历史

| 版本 | 更新内容 |
|------|----------|
| 0.1.0 | 初始版本，基本连接和消息获取 |
| 0.2.0 | 状态管理、错误处理、消息删除、CIS 优化 |

---

## 注意事项

1. **前端为纯静态文件**，无需额外构建步骤
2. **WebSocket** 功能尚未在前端集成（后端已实现）
3. **应用列表** 功能尚未在前端集成（后端已实现）
4. 建议使用 `npm run tauri dev` 进行开发调试