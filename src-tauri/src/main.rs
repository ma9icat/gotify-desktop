#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod gotify;

#[cfg(test)]
mod tests;

use gotify::{GotifyClient, GotifyError, Message};
use serde::{Deserialize, Serialize};
use tauri::{State, api::error::ErrorMessage};
use std::sync::Mutex;

#[derive(Default)]
struct AppState(Mutex<Option<GotifyClient>>);

impl AppState {
    pub fn set_client(&self, client: GotifyClient) {
        *self.0.lock().expect("Failed to lock AppState") = Some(client);
    }

    pub fn clear_client(&self) {
        *self.0.lock().expect("Failed to lock AppState") = None;
    }

    pub fn get_client(&self) -> Result<GotifyClient, GotifyError> {
        self.0
            .lock()
            .expect("Failed to lock AppState")
            .as_ref()
            .cloned()
            .ok_or(GotifyError::NotConnected)
    }
}

#[derive(Serialize, Deserialize)]
struct ConnectRequest {
    server_url: String,
    token: String,
}

#[derive(Serialize, Deserialize)]
struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    error: Option<String>,
}

impl<T> From<Result<T, GotifyError>> for ApiResponse<T> {
    fn from(result: Result<T, GotifyError>) -> Self {
        match result {
            Ok(data) => Self {
                success: true,
                data: Some(data),
                error: None,
            },
            Err(e) => Self {
                success: false,
                data: None,
                error: Some(e.to_string()),
            },
        }
    }
}

#[tauri::command]
async fn connect_to_gotify(state: State<'_, AppState>, req: ConnectRequest) -> ApiResponse<()> {
    let result = GotifyClient::new(&req.server_url, &req.token)
        .map(|client| state.set_client(client));
    result.into()
}

#[tauri::command]
async fn fetch_messages(state: State<'_, AppState>, since: Option<u64>) -> ApiResponse<Vec<Message>> {
    let client = state.get_client()?;
    client.get_messages(since).await.into()
}

#[tauri::command]
async fn disconnect_gotify(state: State<'_, AppState>) -> ApiResponse<()> {
    state.clear_client();
    ApiResponse {
        success: true,
        data: None,
        error: None,
    }
}

#[tauri::command]
async fn delete_message(state: State<'_, AppState>, message_id: u64) -> ApiResponse<()> {
    let client = state.get_client()?;
    client.delete_message(message_id).await.into()
}

fn main() {
    env_logger::init();

    tauri::Builder::default()
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            connect_to_gotify,
            fetch_messages,
            disconnect_gotify,
            delete_message
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
