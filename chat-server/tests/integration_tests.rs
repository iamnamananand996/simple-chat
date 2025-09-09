use chat_server::messages::handle_message;
use chat_server::protocol::{ClientMessage, ServerMessage};
use chat_server::users::{add_user, create_user_stores, remove_user};
use tokio::sync::broadcast;

#[tokio::test]
async fn test_message_handling_functional() {
    // Test message handling with functional approach
    let (users, usernames) = create_user_stores();
    let (broadcast_tx, _) = broadcast::channel(10);

    // Test join message
    let join_msg = ClientMessage::Join {
        username: "test_user".to_string(),
    };

    let response = handle_message(&users, &usernames, &broadcast_tx, None, join_msg).await;

    match response {
        Some(ServerMessage::JoinSuccess { .. }) => {
            // Success
        }
        _ => panic!("Expected JoinSuccess response"),
    }
}

#[tokio::test]
async fn test_user_operations_functional() {
    // Test user operations with functional approach
    let (users, usernames) = create_user_stores();

    // Add user
    let user_id = add_user(&users, &usernames, "alice".to_string()).unwrap();

    // Verify user exists
    assert!(users.contains_key(&user_id));
    assert!(usernames.contains_key("alice"));

    // Remove user
    let removed_username = remove_user(&users, &usernames, user_id);
    assert_eq!(removed_username, Some("alice".to_string()));

    // Verify user is gone
    assert!(!users.contains_key(&user_id));
    assert!(!usernames.contains_key("alice"));
}

#[tokio::test]
async fn test_send_message_functional() {
    // Test sending message with functional approach
    let (users, usernames) = create_user_stores();
    let (broadcast_tx, mut broadcast_rx) = broadcast::channel(10);

    // Add a user first
    let user_id = add_user(&users, &usernames, "sender".to_string()).unwrap();

    // Send a message
    let send_msg = ClientMessage::SendMessage {
        content: "Hello world!".to_string(),
    };

    let response = handle_message(&users, &usernames, &broadcast_tx, Some(user_id), send_msg).await;

    // Should return None for successful message send
    assert!(response.is_none());

    // Check broadcast message
    let broadcast_msg = broadcast_rx.recv().await.unwrap();
    match broadcast_msg {
        ServerMessage::Message { username, content } => {
            assert_eq!(username, "sender");
            assert_eq!(content, "Hello world!");
        }
        _ => panic!("Expected Message broadcast"),
    }
}

#[tokio::test]
async fn test_error_handling_functional() {
    // Test error cases with functional approach
    let (users, usernames) = create_user_stores();
    let (broadcast_tx, _) = broadcast::channel(10);

    // Try to send message without joining
    let send_msg = ClientMessage::SendMessage {
        content: "Should fail".to_string(),
    };

    let response = handle_message(&users, &usernames, &broadcast_tx, None, send_msg).await;

    match response {
        Some(ServerMessage::Error { error }) => {
            assert!(error.contains("Must join"));
        }
        _ => panic!("Expected Error response"),
    }

    // Try duplicate username
    add_user(&users, &usernames, "duplicate".to_string()).unwrap();

    let join_msg = ClientMessage::Join {
        username: "duplicate".to_string(),
    };

    let response = handle_message(&users, &usernames, &broadcast_tx, None, join_msg).await;

    match response {
        Some(ServerMessage::JoinError { error }) => {
            assert!(error.contains("already taken"));
        }
        _ => panic!("Expected JoinError response"),
    }
}
