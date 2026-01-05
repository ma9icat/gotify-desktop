#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod gotify;
mod tests;

use crate::gotify::GotifyClient;
use futures_util::StreamExt;
use log::{debug, error, info, warn};
use std::sync::Mutex;
use tauri::{Emitter, Manager, State};
use tokio::sync::mpsc;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn error(msg: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(msg),
        }
    }

    pub fn from_result(result: Result<T, String>) -> Self {
        match result {
            Ok(data) => Self::success(data),
            Err(msg) => Self::error(msg),
        }
    }
}

struct AppState {
    client: Mutex<Option<GotifyClient>>,
    message_tx: Mutex<Option<mpsc::UnboundedSender<gotify::Message>>>,
    settings: Mutex<AppSettings>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct AppConfig {
    pub servers: Vec<ServerConfig>,
    pub settings: AppSettings,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct AppSettings {
    pub enable_autostart: bool,
    pub minimize_to_tray: bool,
    pub silent_start: bool,
    pub enable_notifications: bool,
    pub log_level: String,
}

impl AppState {
    fn new() -> Self {
        Self {
            client: Mutex::new(None),
            message_tx: Mutex::new(None),
            settings: Mutex::new(AppSettings::default()),
        }
    }

    fn set_client(&self, mut client: GotifyClient, tx: mpsc::UnboundedSender<gotify::Message>) {
        client.set_message_sender(tx.clone());
        *self.client.lock().unwrap() = Some(client);
        *self.message_tx.lock().unwrap() = Some(tx);
        info!("Gotify client initialized");
    }

    fn clear_client(&self) {
        *self.client.lock().unwrap() = None;
        *self.message_tx.lock().unwrap() = None;
        info!("Gotify client cleared");
    }

    fn get_client(&self) -> Result<GotifyClient, String> {
        self.client
            .lock()
            .unwrap()
            .as_ref()
            .cloned()
            .ok_or_else(|| "Not connected to Gotify server. Please connect first.".to_string())
    }

    fn get_settings(&self) -> AppSettings {
        self.settings.lock().unwrap().clone()
    }

    fn set_settings(&self, settings: AppSettings) {
        *self.settings.lock().unwrap() = settings;
    }
}

#[derive(serde::Deserialize)]
struct ConnectRequest {
    server_url: String,
    token: String,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ServerConfig {
    pub id: String,
    pub name: String,
    pub server_url: String,
    pub token: String,
    pub last_used: Option<String>,
}

#[derive(serde::Deserialize)]
struct UpdateConfigRequest {
    id: String,
    name: String,
    server_url: String,
    token: String,
}

fn generate_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    format!(
        "{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis()
    )
}

const APP_CONFIG_DIR: &str = ".gotify-desktop";
const CONFIG_FILE: &str = "config.json";

fn get_config_dir() -> std::path::PathBuf {
    let mut path = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
    path.push(".config");
    path.push(APP_CONFIG_DIR);

    if let Err(e) = std::fs::create_dir_all(&path) {
        error!("Failed to create config directory: {}", e);
    } else {
        info!("Config directory: {:?}", path);
    }

    path
}

fn get_config_path() -> std::path::PathBuf {
    let mut path = get_config_dir();
    path.push(CONFIG_FILE);
    path
}

fn load_app_config() -> AppConfig {
    let config_path = get_config_path();
    if config_path.exists() {
        let content = std::fs::read_to_string(&config_path).unwrap_or_default();
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        AppConfig::default()
    }
}

fn save_app_config(config: &AppConfig) -> Result<(), String> {
    let config_path = get_config_path();
    info!("Saving config to: {:?}", config_path);

    // 确保父目录存在
    if let Some(parent) = config_path.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            error!("Failed to create parent directory: {}", e);
            return Err(format!("Failed to create directory: {}", e));
        }
    }

    let json = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;
    std::fs::write(&config_path, json).map_err(|e| {
        error!("Failed to write config: {}", e);
        e.to_string()
    })
}

// 为了向后兼容，保留这些函数但它们现在操作统一配置
fn load_settings() -> AppSettings {
    load_app_config().settings
}

fn save_settings_to_file(settings: &AppSettings) -> Result<(), String> {
    let mut config = load_app_config();
    config.settings = settings.clone();
    save_app_config(&config)
}

fn load_configs() -> Vec<ServerConfig> {
    load_app_config().servers
}

fn save_configs(configs: &[ServerConfig]) -> Result<(), String> {
    let mut config = load_app_config();
    config.servers = configs.to_owned();
    save_app_config(&config)
}

#[tauri::command]
async fn save_config(
    _app_handle: tauri::AppHandle,
    name: String,
    server_url: String,
    token: String,
) -> Result<ApiResponse<()>, String> {
    let mut configs = load_configs();

    let now = chrono::Utc::now().to_rfc3339();

    let new_config = ServerConfig {
        id: generate_id(),
        name,
        server_url,
        token,
        last_used: Some(now),
    };

    configs.push(new_config);

    save_configs(&configs)?;

    Ok(ApiResponse::success(()))
}

#[tauri::command]
async fn get_configs(
    _app_handle: tauri::AppHandle,
) -> Result<ApiResponse<Vec<ServerConfig>>, String> {
    Ok(ApiResponse::success(load_configs()))
}

#[tauri::command]
async fn delete_config(
    _app_handle: tauri::AppHandle,
    id: String,
) -> Result<ApiResponse<()>, String> {
    let mut configs = load_configs();
    configs.retain(|c| c.id != id);
    save_configs(&configs)?;

    Ok(ApiResponse::success(()))
}

#[tauri::command]
async fn update_config(
    _app_handle: tauri::AppHandle,
    req: UpdateConfigRequest,
) -> Result<ApiResponse<()>, String> {
    let mut configs = load_configs();

    // 更新配置
    if let Some(config) = configs.iter_mut().find(|c| c.id == req.id) {
        config.name = req.name;
        config.server_url = req.server_url;
        config.token = req.token;
    } else {
        return Err("配置不存在".to_string());
    }

    save_configs(&configs)?;

    Ok(ApiResponse::success(()))
}

#[tauri::command]
async fn set_default_config(
    _app_handle: tauri::AppHandle,
    id: String,
) -> Result<ApiResponse<()>, String> {
    let mut configs = load_configs();

    // 更新最后使用时间
    let now = chrono::Utc::now().to_rfc3339();
    if let Some(config) = configs.iter_mut().find(|c| c.id == id) {
        config.last_used = Some(now);
    }

    save_configs(&configs)?;

    Ok(ApiResponse::success(()))
}

#[tauri::command]
async fn get_default_config(
    _app_handle: tauri::AppHandle,
) -> Result<ApiResponse<Option<ServerConfig>>, String> {
    let configs = load_configs();

    // 获取最后使用的配置
    let last_config = configs
        .into_iter()
        .filter(|c| c.last_used.is_some())
        .max_by(|a, b| a.last_used.cmp(&b.last_used));

    Ok(ApiResponse::success(last_config))
}

async fn start_websocket_listener(app_handle: tauri::AppHandle, base_url: String, token: String) {
    info!("Starting WebSocket task...");
    let ws_url = format!(
        "{}/stream?token={}",
        base_url
            .replace("http://", "ws://")
            .replace("https://", "wss://"),
        token
    );
    info!("Connecting to WebSocket: {}", ws_url);

    loop {
        match tokio_tungstenite::connect_async(&ws_url).await {
            Ok((ws_stream, response)) => {
                info!("WebSocket connected, status: {:?}", response.status());
                let (_, mut read) = ws_stream.split();

                while let Some(result) = read.next().await {
                    match result {
                        Ok(msg) => {
                            if msg.is_text() {
                                let text = msg.to_text().unwrap_or("");
                                info!("WebSocket received text: {}", text);
                                if let Ok(value) = serde_json::from_str::<serde_json::Value>(text) {
                                    info!("Parsed JSON: {:?}", value);
                                    if let Some(msg_obj) = value.as_object() {
                                        if let Ok(message) = serde_json::from_value::<gotify::Message>(
                                            serde_json::Value::Object(msg_obj.clone()),
                                        ) {
                                            info!(
                                                "Received message via WebSocket: id={}",
                                                message.id
                                            );
                                            if let Err(e) = app_handle.emit("new-message", &message)
                                            {
                                                error!("Failed to emit message event: {}", e);
                                            }
                                        } else {
                                            warn!("Failed to parse message from JSON");
                                        }
                                    }
                                } else {
                                    warn!("Failed to parse JSON from: {}", text);
                                }
                            } else if msg.is_close() {
                                info!("WebSocket close message received");
                                break;
                            }
                        }
                        Err(e) => {
                            error!("WebSocket error: {}", e);
                            break;
                        }
                    }
                }
                info!("WebSocket connection closed, reconnecting in 5 seconds...");
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            }
            Err(e) => {
                error!(
                    "WebSocket connection failed: {}, retrying in 5 seconds...",
                    e
                );
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            }
        }
    }
}

#[tauri::command]
async fn connect_to_gotify(
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
    req: ConnectRequest,
) -> Result<ApiResponse<()>, String> {
    debug!(
        "Attempting to connect to Gotify server at: {}",
        req.server_url
    );

    match GotifyClient::new(&req.server_url, &req.token) {
        Ok(mut client) => {
            info!("GotifyClient created successfully");

            let (tx, mut rx) = mpsc::unbounded_channel::<gotify::Message>();
            client.set_message_sender(tx.clone());
            info!("Message sender set");

            let base_url = client.get_base_url().to_string();
            let token = client.get_token().to_string();

            let app_clone = app_handle.clone();
            tokio::spawn(async move {
                start_websocket_listener(app_clone, base_url, token).await;
            });

            state.set_client(client, tx);
            info!("Client saved to state");

            let app = app_handle.clone();
            tokio::spawn(async move {
                while let Some(message) = rx.recv().await {
                    info!("Forwarding message to frontend: id={}", message.id);
                    if let Err(e) = app.emit("new-message", message) {
                        error!("Failed to emit message event: {}", e);
                    }
                }
            });

            Ok(ApiResponse::success(()))
        }
        Err(e) => {
            let error_msg = format!("Failed to connect to Gotify server: {}", e);
            error!("{}", error_msg);
            Ok(ApiResponse::error(error_msg))
        }
    }
}

#[tauri::command]
async fn fetch_messages(
    state: State<'_, AppState>,
    since: Option<u64>,
    limit: Option<u64>,
    offset: Option<u64>,
) -> Result<ApiResponse<Vec<gotify::Message>>, String> {
    debug!(
        "fetch_messages called with since: {:?}, limit: {:?}, offset: {:?}",
        since, limit, offset
    );

    match state.get_client() {
        Ok(client) => {
            debug!("Client found, fetching messages...");
            match client.get_messages(since, limit, offset).await {
                Ok(messages) => {
                    info!("Successfully fetched {} messages", messages.len());
                    Ok(ApiResponse::success(messages))
                }
                Err(e) => {
                    error!("Failed to fetch messages: {}", e);
                    Ok(ApiResponse::error(e.to_string()))
                }
            }
        }
        Err(e) => {
            error!("No client found: {}", e);
            Ok(ApiResponse::error(e.to_string()))
        }
    }
}

#[tauri::command]
async fn disconnect_gotify(state: State<'_, AppState>) -> Result<ApiResponse<()>, String> {
    state.clear_client();
    Ok(ApiResponse::success(()))
}

#[tauri::command]
async fn delete_message(
    state: State<'_, AppState>,
    message_id: u64,
) -> Result<ApiResponse<()>, String> {
    match state.get_client() {
        Ok(client) => match client.delete_message(message_id).await {
            Ok(_) => Ok(ApiResponse::success(())),
            Err(e) => Ok(ApiResponse::error(e.to_string())),
        },
        Err(e) => Ok(ApiResponse::error(e.to_string())),
    }
}

#[tauri::command]
async fn get_health(state: State<'_, AppState>) -> Result<ApiResponse<bool>, String> {
    match state.get_client() {
        Ok(client) => match client.get_health().await {
            Ok(healthy) => Ok(ApiResponse::success(healthy)),
            Err(e) => Ok(ApiResponse::error(e.to_string())),
        },
        Err(e) => Ok(ApiResponse::error(e.to_string())),
    }
}

#[tauri::command]
async fn create_message(
    state: State<'_, AppState>,
    title: String,
    message: String,
    priority: i32,
) -> Result<ApiResponse<gotify::Message>, String> {
    match state.get_client() {
        Ok(client) => match client.create_message(&title, &message, priority).await {
            Ok(msg) => Ok(ApiResponse::success(msg)),
            Err(e) => Ok(ApiResponse::error(e.to_string())),
        },
        Err(e) => Ok(ApiResponse::error(e.to_string())),
    }
}

#[tauri::command]
async fn get_applications(
    state: State<'_, AppState>,
) -> Result<ApiResponse<Vec<gotify::Application>>, String> {
    match state.get_client() {
        Ok(client) => match client.get_applications().await {
            Ok(apps) => Ok(ApiResponse::success(apps)),
            Err(e) => Ok(ApiResponse::error(e.to_string())),
        },
        Err(e) => Ok(ApiResponse::error(e.to_string())),
    }
}

// 获取应用设置
#[tauri::command]
async fn get_app_settings(state: State<'_, AppState>) -> Result<ApiResponse<AppSettings>, String> {
    Ok(ApiResponse::success(state.get_settings()))
}

// 更新应用设置
#[tauri::command]
async fn update_app_settings(
    state: State<'_, AppState>,
    settings: AppSettings,
) -> Result<ApiResponse<()>, String> {
    state.set_settings(settings.clone());
    save_settings_to_file(&settings)?;
    Ok(ApiResponse::success(()))
}

// 切换开机启动
#[tauri::command]
async fn toggle_autostart(
    app_handle: tauri::AppHandle,
    enabled: bool,
) -> Result<ApiResponse<bool>, String> {
    use tauri_plugin_autostart::ManagerExt;

    info!("Toggling autostart: enabled={}", enabled);

    let autostart_manager = app_handle.autolaunch();

    if enabled {
        info!("Enabling autostart...");
        if let Err(e) = autostart_manager.enable() {
            error!("Failed to enable autostart: {}", e);
            return Err(format!("Failed to enable autostart: {}", e));
        }
        info!("Autostart enabled successfully");
    } else {
        info!("Disabling autostart...");
        // 禁用开机启动时，如果文件不存在则忽略错误
        if let Err(e) = autostart_manager.disable() {
            // 检查是否是"文件不存在"错误，如果是则忽略
            let error_str = e.to_string();
            if error_str.contains("找不到指定的文件")
                || error_str.contains("The system cannot find the file")
            {
                info!("Autostart file does not exist, skipping disable");
            } else {
                error!("Failed to disable autostart: {}", e);
                return Err(format!("Failed to disable autostart: {}", e));
            }
        }
        info!("Autostart disabled successfully");
    }

    // 返回当前状态
    let is_enabled = autostart_manager.is_enabled().map_err(|e| {
        error!("Failed to check autostart status: {}", e);
        format!("Failed to check autostart status: {}", e)
    })?;

    info!("Autostart status: {}", is_enabled);
    Ok(ApiResponse::success(is_enabled))
}

// 显示窗口
#[tauri::command]
async fn show_window(app: tauri::AppHandle) -> Result<ApiResponse<()>, String> {
    if let Some(window) = app.get_webview_window("main") {
        window.show().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;
    }
    Ok(ApiResponse::success(()))
}

// 隐藏窗口
#[tauri::command]
async fn hide_window(app: tauri::AppHandle) -> Result<ApiResponse<()>, String> {
    if let Some(window) = app.get_webview_window("main") {
        window.hide().map_err(|e| e.to_string())?;
    }
    Ok(ApiResponse::success(()))
}

// 测试 WebSocket 连接
#[tauri::command]
async fn test_websocket(state: State<'_, AppState>) -> Result<ApiResponse<String>, String> {
    match state.get_client() {
        Ok(client) => {
            let ws_url = format!(
                "{}/stream?token={}",
                client
                    .get_base_url()
                    .replace("http://", "ws://")
                    .replace("https://", "wss://"),
                client.get_token()
            );
            info!("Test WebSocket URL: {}", ws_url);
            Ok(ApiResponse::success(ws_url))
        }
        Err(e) => Ok(ApiResponse::error(e)),
    }
}

// 测试事件发送
#[tauri::command]
async fn test_emit_event(app: tauri::AppHandle) -> Result<ApiResponse<String>, String> {
    let test_message = gotify::Message {
        id: 999,
        message: "测试消息".to_string(),
        title: Some("测试标题".to_string()),
        priority: 1,
        timestamp: chrono::Utc::now().to_rfc3339(),
        app_id: 1,
        extras: None,
    };

    match app.emit("new-message", &test_message) {
        Ok(_) => {
            info!("Test event emitted successfully");
            Ok(ApiResponse::success("Event emitted".to_string()))
        }
        Err(e) => {
            error!("Failed to emit test event: {}", e);
            Ok(ApiResponse::error(format!("Failed: {}", e)))
        }
    }
}

// 发送系统通知
#[tauri::command]
async fn send_notification(
    app: tauri::AppHandle,
    title: String,
    body: String,
) -> Result<ApiResponse<()>, String> {
    use tauri_plugin_notification::NotificationExt;
    app.notification()
        .builder()
        .title(title)
        .body(body)
        .show()
        .map_err(|e| e.to_string())?;
    Ok(ApiResponse::success(()))
}

// 检查更新
#[tauri::command]
async fn check_update(app: tauri::AppHandle) -> Result<ApiResponse<serde_json::Value>, String> {
    use tauri_plugin_updater::UpdaterExt;

    let updater = app
        .updater()
        .map_err(|e| format!("获取更新器失败: {}", e))?;

    match updater.check().await {
        Ok(Some(update)) => {
            info!("Update available: version {}", update.version);
            Ok(ApiResponse::success(serde_json::json!({
                "available": true,
                "version": update.version,
                "date": update.date,
                "body": update.body,
                "current_version": env!("CARGO_PKG_VERSION")
            })))
        }
        Ok(None) => {
            info!("No update available");
            Ok(ApiResponse::success(serde_json::json!({
                "available": false,
                "current_version": env!("CARGO_PKG_VERSION")
            })))
        }
        Err(e) => {
            error!("Failed to check update: {}", e);
            Err(format!("检查更新失败: {}", e))
        }
    }
}

// 安装更新
#[tauri::command]
async fn install_update(app: tauri::AppHandle) -> Result<ApiResponse<()>, String> {
    use tauri_plugin_updater::UpdaterExt;

    let updater = app
        .updater()
        .map_err(|e| format!("获取更新器失败: {}", e))?;

    match updater.check().await {
        Ok(Some(update)) => {
            info!("Installing update: version {}", update.version);
            match update.download_and_install(|_, _| {}, || {}).await {
                Ok(_) => {
                    info!("Update installed successfully");
                    Ok(ApiResponse::success(()))
                }
                Err(e) => {
                    error!("Failed to install update: {}", e);
                    Err(format!("安装更新失败: {}", e))
                }
            }
        }
        Ok(None) => Err("没有可用的更新".to_string()),
        Err(e) => {
            error!("Failed to check update: {}", e);
            Err(format!("检查更新失败: {}", e))
        }
    }
}

fn main() {
    // 加载设置
    let settings = load_settings();

    // 初始化日志系统，使用配置中的日志等级
    let log_level = settings.log_level.to_lowercase();
    let level_filter = match log_level.as_str() {
        "trace" => log::LevelFilter::Trace,
        "debug" => log::LevelFilter::Debug,
        "info" => log::LevelFilter::Info,
        "warn" => log::LevelFilter::Warn,
        "error" => log::LevelFilter::Error,
        "off" => log::LevelFilter::Off,
        _ => log::LevelFilter::Info, // 默认 info
    };

    env_logger::Builder::from_default_env()
        .filter_level(level_filter)
        .filter_module("chromium", log::LevelFilter::Error) // 抑制 Chromium 的非错误日志
        .filter_module("ui::gfx", log::LevelFilter::Error) // 抑制 UI/GFX 的 INFO 日志
        .init();

    info!("Loaded settings: {:?}", settings);
    info!("Log level set to: {:?}", level_filter);

    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--hidden"]),
        ))
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(AppState::new())
        .setup(move |app| {
            // 初始化设置到 AppState
            let state: State<AppState> = app.state();
            state.set_settings(settings.clone());

            // 静默启动
            if settings.silent_start {
                let window = app.get_webview_window("main").unwrap();
                window.hide().unwrap();
            }

            // 创建系统托盘
            let show = tauri::menu::MenuItem::with_id(app, "show", "显示窗口", true, None::<&str>)
                .map_err(|e| e.to_string())?;
            let quit = tauri::menu::MenuItem::with_id(app, "quit", "退出", true, None::<&str>)
                .map_err(|e| e.to_string())?;
            let menu =
                tauri::menu::Menu::with_items(app, &[&show, &quit]).map_err(|e| e.to_string())?;

            let _tray = tauri::tray::TrayIconBuilder::with_id("main-tray")
                .menu(&menu)
                .tooltip("Gotify Desktop")
                .icon(app.default_window_icon().unwrap().clone())
                .on_menu_event(move |app, event| match event.id.as_ref() {
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "quit" => {
                        std::process::exit(0);
                    }
                    _ => {}
                })
                .build(app)
                .map_err(|e| e.to_string())?;

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let settings = window.state::<AppState>().get_settings();
                if settings.minimize_to_tray {
                    window.hide().unwrap();
                    api.prevent_close();
                } else {
                    std::process::exit(0);
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            connect_to_gotify,
            fetch_messages,
            disconnect_gotify,
            delete_message,
            get_health,
            create_message,
            get_applications,
            save_config,
            get_configs,
            delete_config,
            set_default_config,
            get_default_config,
            update_config,
            get_app_settings,
            update_app_settings,
            toggle_autostart,
            show_window,
            hide_window,
            send_notification,
            test_websocket,
            test_emit_event,
            check_update,
            install_update
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
