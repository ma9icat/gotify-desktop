#[cfg(test)]
mod tests {
    use crate::gotify::{Application, GotifyClient, GotifyError, Message};
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
        assert_eq!(app.description, "A test application");
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
        let error = GotifyError::AuthFailed("Invalid token".to_string());
        assert_eq!(error.to_string(), "Authentication failed: Invalid token");

        let error = GotifyError::InvalidUrl("not-a-url".to_string());
        assert_eq!(error.to_string(), "Invalid URL: not-a-url");
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

    #[test]
    fn test_message_with_all_priorities() {
        for priority in 0..=5 {
            let json = format!(
                r#"{{
                "id": {},
                "message": "Test message",
                "priority": {},
                "timestamp": "2024-01-01T00:00:00Z",
                "appid": 1
            }}"#,
                priority, priority
            );

            let message: Message = serde_json::from_str(&json).unwrap();
            assert_eq!(message.priority, priority);
        }
    }

    #[test]
    fn test_message_serialization() {
        let message = Message {
            id: 1,
            message: "Test".to_string(),
            title: Some("Title".to_string()),
            priority: 3,
            timestamp: "2024-01-01T00:00:00Z".to_string(),
            app_id: 1,
            extras: Some(serde_json::json!({"key": "value"})),
        };

        let json = serde_json::to_string(&message).unwrap();
        let deserialized: Message = serde_json::from_str(&json).unwrap();
        assert_eq!(message.id, deserialized.id);
        assert_eq!(message.message, deserialized.message);
    }
}
