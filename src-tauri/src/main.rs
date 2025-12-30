#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use gotify::{GotifyClient, GotifyError};
use log::info;
use std::sync::Mutex;
use tauri::{State, command};

struct AppState {
    client: Mutex<Option<GotifyClient>>,
}

impl AppState {
    fn new() -> Self {
        Self {
            client: Mutex::new(None),
        }
    }

    fn set_client(&self, client: GotifyClient) {
        *self.client.lock().unwrap() = Some(client);
        info!("Gotify client initialized");
    }

    fn clear_client(&self) {
        *self.client.lock().unwrap() = None;
        info!("Gotify client cleared");
    }

    fn get_client(&self) -> Result<GotifyClient, String> {
        self.client
            .lock()
            .unwrap()
            .clone()
            .ok_or_else(|| "Not connected to Gotify server. Please connect first.".to_string())
    }
}

#[derive(serde::Deserialize)]
struct ConnectRequest {
    server_url: String,
    token: String,
}

type ApiResponse<T> = Result<T, String>;

#[command]
async fn connect_to_gotify(state: State<'_, AppState>, req: ConnectRequest) -> ApiResponse<()> {
    info!("Attempting to connect to Gotify server at: {}", req.server_url);

    match GotifyClient::new(&req.server_url, &req.token) {
        Ok(client) => {
            state.set_client(client);
            Ok(())
        }
        Err(e) => {
            let error_msg = format!("Failed to connect to Gotify server: {}", e);
            error!("{}", error_msg);
            Err(error_msg)
        }
    }
}

#[command]
async fn fetch_messages(state: State<'_, AppState>, since: Option<u64>) -> ApiResponse<Vec<gotify::Message>> {
    let client = state.get_client()?;
    client.get_messages(since).await.map_err(|e| e.to_string())
}

#[command]
async fn disconnect_gotify(state: State<'_, AppState>) -> ApiResponse<()> {
    state.clear_client();
    Ok(())
}

#[command]
async fn delete_message(state: State<'_, AppState>, message_id: u64) -> ApiResponse<()> {
    let client = state.get_client()?;
    client.delete_message(message_id).await.map_err(|e| e.to_string())
}

#[command]
async fn get_health(state: State<'_, AppState>) -> ApiResponse<bool> {
    let client = state.get_client()?;
    client.get_health().await.map_err(|e| e.to_string())
}

#[command]
async fn create_message(state: State<'_, AppState>, title: String, message: String, priority: i32) -> ApiResponse<gotify::Message> {
    let client = state.get_client()?;
    client.create_message(&title, &message, priority).await.map_err(|e| e.to_string())
}

#[command]
async fn get_applications(state: State<'_, AppState>) -> ApiResponse<Vec<gotify::Application>> {
    let client = state.get_client()?;
    client.get_applications().await.map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            connect_to_gotify,
            fetch_messages,
            disconnect_gotify,
            delete_message,
            get_health,
            create_message,
            get_applications
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}