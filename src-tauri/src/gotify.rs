use log::info;
use reqwest::{Client, Error as ReqwestError};
use serde::Serialize;
use serde_json::Error as JsonError;
use thiserror::Error;
use tokio::sync::mpsc;
use url::Url;

#[derive(Debug, Error)]
pub enum GotifyError {
    #[error("Authentication failed: {0}")]
    AuthFailed(String),
    #[error("Server error: {0}")]
    ServerError(String),
    #[error("Resource not found: {0}")]
    NotFound(String),
    #[error("Network error: {0}")]
    NetworkError(#[from] ReqwestError),
    #[error("JSON parsing error: {0}")]
    JsonError(#[from] JsonError),
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),
    #[error("Request failed: {0}")]
    RequestError(String),
}

impl GotifyError {
    fn from_status_code(status: reqwest::StatusCode, body: String) -> Self {
        match status {
            reqwest::StatusCode::UNAUTHORIZED => Self::AuthFailed(body),
            reqwest::StatusCode::FORBIDDEN => {
                Self::AuthFailed("Access forbidden. Check your token permissions.".to_string())
            }
            reqwest::StatusCode::NOT_FOUND => Self::NotFound(body),
            reqwest::StatusCode::INTERNAL_SERVER_ERROR
            | reqwest::StatusCode::BAD_GATEWAY
            | reqwest::StatusCode::SERVICE_UNAVAILABLE => Self::ServerError(body),
            _ => Self::RequestError(format!("HTTP {}: {}", status, body)),
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct Message {
    pub id: u64,
    pub message: String,
    pub title: Option<String>,
    pub priority: i32,
    #[serde(alias = "date")]
    pub timestamp: String,
    #[serde(alias = "appid")]
    pub app_id: u64,
    pub extras: Option<serde_json::Value>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct Application {
    pub id: u64,
    pub name: String,
    pub description: String,
    pub token: Option<String>,
}

#[derive(Debug, Clone)]
pub struct GotifyClient {
    base_url: String,
    token: String,
    client: Client,
    // 用于发送新消息的通道发送端
    message_tx: Option<mpsc::UnboundedSender<Message>>,
}

impl GotifyClient {
    pub fn new(server_url: &str, token: &str) -> Result<Self, GotifyError> {
        let base_url = server_url.trim_end_matches('/').to_string();

        // Validate URL
        match Url::parse(&base_url) {
            Ok(_) => {}
            Err(e) => return Err(GotifyError::InvalidUrl(e.to_string())),
        }

        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(GotifyError::NetworkError)?;

        Ok(Self {
            base_url,
            token: token.to_string(),
            client,
            message_tx: None,
        })
    }

    pub fn set_message_sender(&mut self, tx: mpsc::UnboundedSender<Message>) {
        self.message_tx = Some(tx);
    }

    pub fn get_base_url(&self) -> &str {
        &self.base_url
    }

    pub fn get_token(&self) -> &str {
        &self.token
    }

    async fn get(&self, endpoint: &str) -> Result<reqwest::Response, GotifyError> {
        let url = if endpoint.contains('?') {
            format!("{}/{}&token={}", self.base_url, endpoint, self.token)
        } else {
            format!("{}/{}?token={}", self.base_url, endpoint, self.token)
        };
        info!("GET {}", url);
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(GotifyError::NetworkError)?;
        Ok(resp)
    }

    async fn delete(&self, endpoint: &str) -> Result<(), GotifyError> {
        let url = if endpoint.contains('?') {
            format!("{}/{}&token={}", self.base_url, endpoint, self.token)
        } else {
            format!("{}/{}?token={}", self.base_url, endpoint, self.token)
        };
        info!("DELETE {}", url);
        self.client
            .delete(&url)
            .send()
            .await
            .map_err(GotifyError::NetworkError)?;
        Ok(())
    }

    async fn post(&self, endpoint: &str, body: &str) -> Result<reqwest::Response, GotifyError> {
        let url = if endpoint.contains('?') {
            format!("{}/{}&token={}", self.base_url, endpoint, self.token)
        } else {
            format!("{}/{}?token={}", self.base_url, endpoint, self.token)
        };
        info!("POST {}", url);
        let resp = self
            .client
            .post(&url)
            .body(body.to_string())
            .send()
            .await
            .map_err(GotifyError::NetworkError)?;
        Ok(resp)
    }

    async fn handle_response(resp: reqwest::Response) -> Result<String, GotifyError> {
        let status = resp.status();
        let body = resp.text().await.map_err(GotifyError::NetworkError)?;

        if !status.is_success() {
            return Err(GotifyError::from_status_code(status, body));
        }

        Ok(body)
    }

    pub async fn get_messages(
        &self,
        since: Option<u64>,
        limit: Option<u64>,
        offset: Option<u64>,
    ) -> Result<Vec<Message>, GotifyError> {
        let mut endpoint = "message".to_string();
        let mut params = Vec::new();

        if let Some(id) = since {
            params.push(format!("since={}", id));
        }
        if let Some(l) = limit {
            params.push(format!("limit={}", l));
        }
        if let Some(o) = offset {
            params.push(format!("offset={}", o));
        }

        if !params.is_empty() {
            endpoint = format!("message?{}", params.join("&"));
        }

        info!("Fetching messages from endpoint: {}", endpoint);

        let resp = self.get(&endpoint).await?;
        let body = Self::handle_response(resp).await?;
        info!("Got response body: {}", body);

        let value: serde_json::Value =
            serde_json::from_str(&body).map_err(GotifyError::JsonError)?;
        info!("Parsed JSON: {}", value);

        let messages: Vec<Message> = value["messages"]
            .as_array()
            .ok_or_else(|| {
                let err = serde_json::json!({"error": "no messages array in response"});
                let err_str = err.to_string();
                GotifyError::JsonError(serde::ser::Error::custom(err_str))
            })?
            .iter()
            .filter_map(|m| serde_json::from_value(m.clone()).ok())
            .collect();

        info!("Parsed {} messages", messages.len());
        Ok(messages)
    }

    pub async fn delete_message(&self, message_id: u64) -> Result<(), GotifyError> {
        let endpoint = format!("message/{}", message_id);
        self.delete(&endpoint).await?;
        Ok(())
    }

    pub async fn create_message(
        &self,
        title: &str,
        message: &str,
        priority: i32,
    ) -> Result<Message, GotifyError> {
        #[derive(Serialize)]
        struct Request {
            title: String,
            message: String,
            priority: i32,
        }

        let req_body = serde_json::to_string(&Request {
            title: title.to_string(),
            message: message.to_string(),
            priority,
        })
        .map_err(GotifyError::JsonError)?;

        let resp = self.post("message", &req_body).await?;
        let body = Self::handle_response(resp).await?;
        let value: serde_json::Value =
            serde_json::from_str(&body).map_err(GotifyError::JsonError)?;
        let msg: Message = serde_json::from_value(value).map_err(GotifyError::JsonError)?;
        Ok(msg)
    }

    pub async fn get_applications(&self) -> Result<Vec<Application>, GotifyError> {
        let resp = self.get("application").await?;
        let body = Self::handle_response(resp).await?;
        let value: serde_json::Value =
            serde_json::from_str(&body).map_err(GotifyError::JsonError)?;

        let apps: Vec<Application> =
            serde_json::from_value(value).map_err(GotifyError::JsonError)?;
        Ok(apps)
    }

    pub async fn get_health(&self) -> Result<bool, GotifyError> {
        let url = format!("{}/health", self.base_url);
        info!("GET {}", url);
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(GotifyError::NetworkError)?;
        Ok(resp.status().is_success())
    }
}
