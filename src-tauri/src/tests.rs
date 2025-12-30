#[cfg(test)]
mod tests {
    use crate::gotify::{GotifyClient, GotifyError, Message, Application};
    use serde_json;

    #[test]
    fn test_message_deserialization() {
        let json = r#"{
            "id": 1,
            "message": "Test message",
            "title": "Test title",
            "priority": 5,
            "timestamp": "2024-01-01T00:00:00Z",
            "appid": 1,
            "extras": {"foo": "bar"}
        }"#;

        let message: Message = serde_json::from_str(json).unwrap();
        assert_eq!(message.id, 1);
        assert_eq!(message.message, "Test message");
        assert_eq!(message.title, Some("Test title".to_string()));
        assert_eq!(message.priority, 5);
        assert_eq!(message.app_id, 1);
        assert!(message.extras.is_some());
    }

    #[test]
    fn test_application_deserialization() {
        let json = r#"{
            "id": 1,
            "name": "Test App",
            "description": "A test application",
            "token": "abc123"
        }"#;

        let app: Application = serde_json::from_str(json).unwrap();
        assert_eq!(app.id, 1);
        assert_eq!(app.name, "Test App");
        assert_eq!(app.description, Some("A test application".to_string()));
        assert_eq!(app.token, Some("abc123".to_string()));
    }

    #[test]
    fn test_message_without_title() {
        let json = r#"{
            "id": 2,
            "message": "Message without title",
            "priority": 0,
            "timestamp": "2024-01-01T00:00:00Z",
            "appid": 1
        }"#;

        let message: Message = serde_json::from_str(json).unwrap();
        assert_eq!(message.title, None);
    }

    #[test]
    fn test_gotify_error_display() {
        let error = GotifyError::AuthFailed;
        assert_eq!(error.to_string(), "Authentication failed");

        let error = GotifyError::NotConnected;
        assert_eq!(error.to_string(), "Not connected to Gotify server");
    }

    #[test]
    fn test_invalid_url_error() {
        let result = GotifyClient::new("not-a-valid-url", "token");
        assert!(result.is_err());
        if let Err(GotifyError::InvalidUrl(_)) = result {
            // Expected error type
        } else {
            panic!("Expected InvalidUrl error");
        }
    }
}