# Gotify Desktop 项目上下文文档

## 项目概述

Gotify Desktop 是一个跨平台的 Gotify 服务器桌面客户端应用，使用 Rust + Tauri 2.x 框架开发。该项目旨在为用户提供一个原生桌面应用来连接和交互 Gotify 消息服务器，支持实时消息推送、消息管理、多服务器配置等功能。

**仓库地址**: https://github.com/ma9icat/gotify-desktop

**技术栈：**
- **前端**：原生 HTML5/CSS3 + JavaScript (ES6+)（src/ 目录）
- **后端/框架**：Tauri 2.x（Rust） + Rust crates（src-tauri/ 目录）
- **通信**：HTTP REST API + WebSocket（实时消息推送）
- **异步运行时**：Tokio 1.x
- **HTTP 客户端**：reqwest 0.12
- **WebSocket**：tokio-tungstenite 0.26

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
npm run dev

# 构建生产版本（带优化）
npm run build

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
│   ├── build.rs                 # 构建脚本
│   ├── capabilities/            # Tauri 2.x 权限配置
│   │   └── default.json
│   └── icons/                   # 应用图标
│       ├── icon.ico
│       └── icon.png
├── .github/workflows/           # CI/CD 配置
│   └── ci.yml                   # GitHub Actions 工作流
├── package.json                 # Node.js 依赖和脚本
└── README.md                    # 项目说明
```

---

## 核心功能

### 已实现功能

#### 1. 服务器连接管理 (`connect_to_gotify` 命令)
- 输入 Gotify 服务器 URL 和 Token
- URL 格式验证
- 建立持久化的客户端连接（存储在 `AppState` 中）
- 统一 API 响应格式 (`ApiResponse<T>`)
- 支持 WebSocket 实时消息推送

#### 2. 消息获取与显示 (`fetch_messages` 命令)
- 获取消息列表（支持分页）
- 支持增量获取（`since` 参数用于获取新消息）
- 支持分页参数（`limit`, `offset`）
- 实时消息推送（通过 WebSocket）
- 消息优先级颜色标识

#### 3. 消息管理 (`delete_message` 命令)
- 按 ID 删除单条消息
- 删除后自动更新消息列表

#### 4. 多服务器配置管理
- **保存配置** (`save_config`): 保存服务器 URL、Token 和名称
- **获取配置列表** (`get_configs`): 获取所有已保存的配置
- **删除配置** (`delete_config`): 删除指定配置
- **更新配置** (`update_config`): 编辑现有配置
- **设置默认配置** (`set_default_config`): 设置最后使用的配置
- **获取默认配置** (`get_default_config`): 自动连接最后使用的服务器
- 配置持久化存储在用户目录 `~/.config/.gotify-desktop/config.json`

#### 5. 应用设置
- **开机启动** (`toggle_autostart`): 应用随系统自动启动
- **托盘运行** (`minimize_to_tray`): 关闭窗口时最小化到系统托盘
- **静默启动** (`silent_start`): 启动时不显示主窗口
- **系统通知** (`enable_notifications`): 收到新消息时弹出系统通知
- 设置持久化存储在用户目录 `~/.config/.gotify-desktop/settings.json`

#### 6. 系统通知 (`send_notification` 命令)
- 新消息到达时弹出系统通知
- 支持自定义标题和内容

#### 7. 窗口管理
- **显示窗口** (`show_window`): 显示并聚焦主窗口
- **隐藏窗口** (`hide_window`): 隐藏主窗口
- 侧边栏折叠/展开功能

#### 8. 系统托盘
- **托盘图标**：应用启动时自动创建系统托盘图标
- **托盘菜单**：包含"显示窗口"和"退出"选项
- **窗口关闭行为**：当 `minimize_to_tray` 设置启用时，关闭窗口会最小化到托盘而不是退出应用

#### 9. 健康检查 (`get_health` 方法)
- 检查 Gotify 服务器健康状态

#### 10. 应用列表管理 (`get_applications` 命令)
- 获取已注册的应用列表

#### 11. 创建消息 (`create_message` 命令)
- 通过 API 创建新消息

#### 12. 测试命令
- **test_websocket**: 测试 WebSocket 连接，返回 WebSocket URL（用于调试）
- **test_emit_event**: 测试事件发送到前端（用于调试）

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
    connected: boolean,         // 连接状态
    serverUrl: string,          // 服务器地址
    messages: Message[],        // 消息列表
    loading: boolean,           // 加载状态
    error: string | null,       // 错误信息
    loadingMore: boolean,       // 加载更多状态
    hasMoreMessages: boolean,   // 是否有更多消息
    configs: ServerConfig[],    // 服务器配置列表
    currentConfigId: string,    // 当前连接的配置 ID
    settings: AppSettings,      // 应用设置
    currentPage: string,        // 当前页面
    sidebarCollapsed: boolean   // 侧边栏折叠状态
};
```

**核心函数：**
- `updateUIState(updates)` - 统一更新 UI 状态
- `render()` - 渲染整个界面
- `renderMessages()` - 渲染消息列表
- `renderConfigs()` - 渲染服务器配置列表
- `refreshMessages()` - 刷新消息（带错误处理）
- `loadMoreMessages()` - 加载更多消息（分页）
- `deleteMessage(id)` - 删除消息
- `connectToServer(url, token, name, id)` - 连接到服务器
- `autoConnectDefault()` - 自动连接默认配置
- `loadConfigs()` - 加载配置列表
- `loadAppSettings()` - 加载应用设置
- `saveAppSettings()` - 保存应用设置
- `switchPage(pageName)` - 切换页面
- `toggleSidebar()` - 切换侧边栏
- `escapeHtml()` - HTML 转义（防止 XSS）
- `formatTime()` - 时间格式化

**事件处理：**
- 连接按钮点击 → `connect_to_gotify`
- 断开连接按钮 → `disconnect_gotify`
- 刷新按钮 → `refreshMessages`
- 删除按钮 → `delete_message`
- WebSocket 事件 → `new-message`（实时推送）
- 页面导航 → `switchPage`
- 侧边栏折叠 → `toggleSidebar`

### 后端 (src-tauri/src/)

**main.rs:**

**`AppState` 结构：**
```rust
struct AppState {
    client: Mutex<Option<GotifyClient>>,
    message_tx: Mutex<Option<mpsc::UnboundedSender<Message>>>,
    settings: Mutex<AppSettings>,
}
```

**`AppSettings` 结构：**
```rust
pub struct AppSettings {
    pub enable_autostart: bool,
    pub minimize_to_tray: bool,
    pub silent_start: bool,
    pub enable_notifications: bool,
}
```

**`ServerConfig` 结构：**
```rust
pub struct ServerConfig {
    pub id: String,
    pub name: String,
    pub server_url: String,
    pub token: String,
    pub last_used: Option<String>,
}
```

**核心方法：**
- `AppState::new()` - 创建新状态
- `AppState::set_client()` - 设置客户端
- `AppState::clear_client()` - 清除客户端
- `AppState::get_client()` - 获取客户端
- `AppState::get_settings()` - 获取设置
- `AppState::set_settings()` - 更新设置
- `start_websocket_listener()` - WebSocket 监听和自动重连
- `get_config_dir()` - 获取配置目录路径
- `get_config_path()` - 获取配置文件路径

**Tauri 命令：**
| 命令 | 参数 | 返回 | 说明 |
|------|------|------|------|
| `connect_to_gotify` | `{ server_url: string, token: string }` | `ApiResponse<()>` | 连接到 Gotify 服务器（支持 WebSocket） |
| `fetch_messages` | `since: Option<u64>, limit: Option<u64>, offset: Option<u64>` | `ApiResponse<Vec<Message>>` | 获取消息列表（支持分页和增量获取） |
| `disconnect_gotify` | 无 | `ApiResponse<()>` | 断开连接 |
| `delete_message` | `messageId: u64` | `ApiResponse<()>` | 删除消息 |
| `get_health` | 无 | `ApiResponse<bool>` | 健康检查 |
| `create_message` | `title: string, message: string, priority: i32` | `ApiResponse<Message>` | 创建消息 |
| `get_applications` | 无 | `ApiResponse<Vec<Application>>` | 获取应用列表 |
| `save_config` | `name: string, server_url: string, token: string` | `ApiResponse<()>` | 保存配置 |
| `get_configs` | 无 | `ApiResponse<Vec<ServerConfig>>` | 获取配置列表 |
| `delete_config` | `id: string` | `ApiResponse<()>` | 删除配置 |
| `update_config` | `id: string, name: string, server_url: string, token: string` | `ApiResponse<()>` | 更新配置 |
| `set_default_config` | `id: string` | `ApiResponse<()>` | 设置默认配置 |
| `get_default_config` | 无 | `ApiResponse<Option<ServerConfig>>` | 获取默认配置 |
| `get_app_settings` | 无 | `ApiResponse<AppSettings>` | 获取应用设置 |
| `update_app_settings` | `settings: AppSettings` | `ApiResponse<()>` | 更新应用设置 |
| `toggle_autostart` | `enabled: bool` | `ApiResponse<bool>` | 切换开机启动 |
| `show_window` | 无 | `ApiResponse<()>` | 显示窗口 |
| `hide_window` | 无 | `ApiResponse<()>` | 隐藏窗口 |
| `send_notification` | `title: string, body: string` | `ApiResponse<()>` | 发送系统通知 |
| `test_websocket` | 无 | `ApiResponse<String>` | 测试 WebSocket 连接（返回 WebSocket URL） |
| `test_emit_event` | 无 | `ApiResponse<String>` | 测试事件发送（用于调试） |

**gotify.rs:**

**GotifyError 错误类型:**
```rust
pub enum GotifyError {
    NotConnected,
    AuthFailed(String),
    ServerError(String),
    NotFound(String),
    NetworkError(ReqwestError),
    JsonError(JsonError),
    InvalidUrl(String),
    RequestError(String),
    Unknown(String),
}
```

**GotifyClient 结构：**
```rust
pub struct GotifyClient {
    base_url: String,
    token: String,
    client: Client,
    message_tx: Option<mpsc::UnboundedSender<Message>>,
}
```

**GotifyClient 方法:**
| 方法 | 说明 |
|------|------|
| `new(url, token)` | 创建客户端（带 30 秒超时） |
| `set_message_sender(tx)` | 设置消息发送器（用于 WebSocket） |
| `start_websocket()` | 启动 WebSocket 实时消息推送 |
| `get_messages(since, limit, offset)` | 获取消息列表（支持分页） |
| `delete_message(id)` | 删除消息 |
| `create_message(title, message, priority)` | 创建消息 |
| `get_applications()` | 获取应用列表 |
| `get_health()` | 健康检查 |
| `get(endpoint)` | GET 请求 |
| `delete(endpoint)` | DELETE 请求 |
| `post(endpoint, body)` | POST 请求 |
| `handle_response(resp)` | 统一 HTTP 响应处理 |

**tests.rs:**
- 消息/应用反序列化测试
- 错误类型 Display 测试
- URL 验证测试

---

## API 参考

### 统一响应格式

```rust
struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
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
    pub extras: Option<serde_json::Value>,
}
```

**Application**
```rust
pub struct Application {
    pub id: u64,
    pub name: String,
    pub description: String,
    pub token: Option<String>,
}
```

---

## 配置文件

### 配置存储路径

- **Windows**: `C:\Users\<用户名>\.config\.gotify-desktop\`
- **Linux/macOS**: `~/.config/.gotify-desktop/`

**配置文件：**
- `config.json` - 统一配置文件（包含服务器配置列表和应用设置）

### Tauri 配置 (tauri.conf.json)

**窗口配置：**
- 标题：Gotify Desktop
- 大小：900x700（最小 600x400）
- 可调整大小：是
- 居中显示：是

**开发配置：**
- 开发服务器：`http://localhost:1420`
- 开发前命令：`serve src -l 1420`

### 权限配置 (capabilities/default.json)

**允许的权限：**
- `core:default` - 核心 Tauri 功能
- `core:event:allow-listen` - 事件监听
- `core:event:allow-emit` - 事件发送
- `store:allow-load` - 加载存储
- `store:allow-save` - 保存存储

---

## CI/CD 配置

**.github/workflows/ci.yml:**

**Jobs:**
1. **test**
   - 运行 Rust 测试
   - 代码格式检查
   - Clippy 静态分析

2. **build**
   - 构建 Tauri 应用
   - 上传构建产物

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

[profile.dev]
debug = true

[profile.dev.package."*"]
opt-level = 0
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
- 权限：通过 capabilities 系统管理
- CSP：未限制

### 状态管理

- **后端**：`AppState` (Mutex<GotifyClient, message_tx, settings>)
- **前端**：`AppState` 对象（集中管理所有 UI 状态）

### 测试

- **单元测试**：`src-tauri/src/tests.rs`
- 运行方式：`cd src-tauri && cargo test`

### 代码优化

- **WebSocket 重连**：提取为独立函数 `start_websocket_listener`，支持自动重连（5秒间隔）
- **配置路径管理**：使用常量 `APP_CONFIG_DIR` 和 `CONFIG_FILE` 管理路径
- **错误处理**：移除未使用的错误变体，简化错误类型
- **测试覆盖**：添加消息序列化、优先级测试等用例

---

## 待实现功能

1. **暗色模式**
   - CSS 变量主题切换
   - 持久化主题偏好

2. **应用列表管理前端集成**
   - 显示和管理应用
   - 应用信息查看

3. **消息搜索**
   - 按关键词搜索消息
   - 按日期范围筛选

4. **消息导出**
   - 导出为 JSON/CSV 格式

5. **消息详情查看**
   - 查看完整消息内容
   - 显示消息元数据

---

## 版本历史

| 版本 | 更新内容 |
|------|----------|
| 0.1.0 | 初始版本，基本连接和消息获取 |
| 0.2.0 | WebSocket 实时消息、多服务器配置、应用设置、系统通知、系统托盘、侧边栏折叠 |
| 0.3.0 | 代码优化：WebSocket 重连逻辑提取、配置路径管理优化、测试覆盖率提升 |

---

## 注意事项

1. **前端为纯静态文件**，无需额外构建步骤
2. **WebSocket 实时消息已实现**并已在前端集成（自动重连）
3. **多服务器配置已实现**支持保存、编辑、删除配置
4. **应用设置已实现**包括开机启动、托盘运行、静默启动、系统通知
5. **系统托盘已实现**包含托盘图标和菜单
6. 建议使用 `npm run dev` 进行开发调试
7. 配置文件存储在用户目录下的 `.config/.gotify-desktop/config.json`
8. 侧边栏支持折叠，状态会保存到 localStorage