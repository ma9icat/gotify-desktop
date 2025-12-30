use serde::{Deserialize, Serialize};
use url::Url;
use reqwest::{Client, StatusCode};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GotifyError {
    #[error("Invalid URL: {0}")]
    InvalidUrl(#[from] url::ParseError),
    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),
    #[error("JSON parse error: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("Server returned error: {0}")]
    ServerError(String),
    #[error("Not connected to Gotify server")]
    NotConnected,
    #[error("Authentication failed")]
    AuthFailed,
    #[error("Resource not found: {0}")]
    NotFound(String),
    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl From<GotifyError> for String {
    fn from(e: GotifyError) -> Self {
        e.to_string()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Message {
    pub id: u64,
    pub message: String,
    pub title: Option<String>,
    pub priority: i32,
    pub timestamp: String,
    #[serde(rename = "appid")]
    pub app_id: u64,
    pub extras: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Application {
    pub id: u64,
    pub name: String,
    pub description: Option<String>,
    pub token: Option<String>,
    pub internal: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateMessageRequest {
    pub message: String,
    pub title: Option<String>,
    pub priority: Option<i32>,
    pub extras: Option<serde_json::Value>,
}

#[derive(Clone)]
pub struct GotifyClient {
    base_url: Url,
    token: String,
    client: Client,
}

impl GotifyClient {
    pub fn new(base_url: &str, token: &str) -> Result<Self, GotifyError> {
        let url = Url::parse(base_url)?;
        Ok(Self {
            base_url: url,
            token: token.to_string(),
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()?,
        })
    }

    fn request(&self) -> reqwest::RequestBuilder {
        self.client
            .get(self.base_url.clone())
            .header("X-Gotify-Key", &self.token)
    }

    pub async fn get_messages(&self, since: Option<u64>) -> Result<Vec<Message>, GotifyError> {
        let mut url = self.base_url.join("/message")?;
        if let Some(since_id) = since {
            url.query_pairs_mut().append_pair("since", &since_id.to_string());
        }
        let resp = self
            .client
            .get(url)
            .header("X-Gotify-Key", &self.token)
            .send()
            .await?;

        Self::handle_response(resp).await?.json::<serde_json::Value>().await
            .and_then(|v| {
                let messages = v["messages"]
                    .as_array()
                    .ok_or_else(|| serde_json::Error::custom("no messages array"))?;
                messages.iter()
                    .map(|m| serde_json::from_value(m.clone()))
                    .collect()
            })
    }

    pub async fn delete_message(&self, message_id: u64) -> Result<(), GotifyError> {
        let url = self.base_url.join(&format!("/message/{}", message_id))?;
        let resp = self
            .client
            .delete(url)
            .header("X-Gotify-Key", &self.token)
            .send()
            .await?;

        Self::handle_response(resp).await?;
        Ok(())
    }

    pub async fn create_message(&self, request: CreateMessageRequest) -> Result<Message, GotifyError> {
        let url = self.base_url.join("/message")?;
        let resp = self
            .client
            .post(url)
            .header("X-Gotify-Key", &self.token)
            .json(&request)
            .send()
            .await?;

        Self::handle_response(resp).await?.json::<Message>().await
            .map_err(|e| GotifyError::JsonError(e))
    }

    pub async fn get_applications(&self) -> Result<Vec<Application>, GotifyError> {
        let url = self.base_url.join("/application")?;
        let resp = self
            .client
            .get(url)
            .header("X-Gotify-Key", &self.token)
            .send()
            .await?;

        Self::handle_response(resp).await?.json::<Vec<Application>>().await
            .map_err(|e| GotifyError::JsonError(e))
    }

    pub async fn get_health(&self) -> Result<bool, GotifyError> {
        let url = self.base_url.join("/health")?;
        let resp = self
            .client
            .get(url)
            .send()
            .await?;

        match resp.status() {
            StatusCode::OK => Ok(true),
            _ => Ok(false),
        }
    }

    async fn handle_response(resp: reqwest::Response) -> Result<reqwest::Response, GotifyError> {
        match resp.status() {
            StatusCode::OK | StatusCode::CREATED => Ok(resp),
            StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => {
                Err(GotifyError::AuthFailed)
            }
            StatusCode::NOT_FOUND => {
                let url = resp.url().clone();
                Err(GotifyError::NotFound(url.to_string()))
            }
            StatusCode::INTERNAL_SERVER_ERROR | StatusCode::BAD_GATEWAY => {
                let error_text = resp.text().await.unwrap_or_default();
                Err(GotifyError::ServerError(error_text))
            }
            StatusCode::TOO_MANY_REQUESTS => {
                Err(GotifyError::ServerError("Rate limited".to_string()))
            }
            _ => {
                let error_text = resp.text().await.unwrap_or_default();
                Err(GotifyError::Unknown(error_text))
            }
        }
    }
}
