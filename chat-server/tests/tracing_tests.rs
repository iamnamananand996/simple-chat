#![allow(clippy::collapsible_match)]

use chat_server::protocol::{ClientMessage, ServerMessage};
use chat_server::websocket::run_chat_server;
use futures_util::{SinkExt, StreamExt};
use std::time::Duration;
use tokio::time::timeout;
use tokio_tungstenite::{connect_async, tungstenite::Message};

#[tokio::test]
async fn test_detailed_message_tracing() {
    let addr = "127.0.0.1:9993";

    // Start server in background
    let server_handle = tokio::spawn(async move {
        run_chat_server(addr).await.unwrap();
    });

    // Give server time to start
    tokio::time::sleep(Duration::from_millis(100)).await;

    println!("\n=== DETAILED MESSAGE TRACING TEST ===");

    // Create 3 test users
    let users = vec![
        ("alice", "Hello everyone!"),
        ("bob", "Hi Alice! How are you?"),
        ("charlie", "Good morning chat!"),
    ];

    let mut user_connections = vec![];

    // Connect all users first
    for (username, _) in &users {
        let ws_url = format!("ws://{addr}");
        let (ws_stream, _) = connect_async(ws_url).await.unwrap();
        let (mut ws_sender, mut ws_receiver) = ws_stream.split();

        // Join the chat
        let join_msg = ClientMessage::Join {
            username: username.to_string(),
        };
        let json = join_msg.to_json().unwrap();
        ws_sender.send(Message::Text(json)).await.unwrap();
        println!("[TEST] {username} sent join request");

        // Read join response
        if let Ok(Some(Ok(msg))) = timeout(Duration::from_secs(1), ws_receiver.next()).await {
            if let Message::Text(response) = msg {
                println!("[TEST] {} received: {}", username, response.trim());
            }
        }

        user_connections.push((username, ws_sender, ws_receiver));
    }

    // Give time for all join notifications to propagate
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Have each user send their message
    for (i, (username, message)) in users.iter().enumerate() {
        let (_, ws_sender, _) = &mut user_connections[i];

        let send_msg = ClientMessage::SendMessage {
            content: message.to_string(),
        };
        let json = send_msg.to_json().unwrap();
        ws_sender.send(Message::Text(json)).await.unwrap();
        println!("[TEST] {username} sent message: '{message}'");

        // Give time for message to be processed and broadcast
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Try to read broadcasts on all other users
        for (j, (other_user, _, ws_receiver)) in user_connections.iter_mut().enumerate() {
            if i != j {
                // Don't read on the sender
                if let Ok(Some(Ok(msg))) =
                    timeout(Duration::from_millis(100), ws_receiver.next()).await
                {
                    if let Message::Text(response) = msg {
                        println!(
                            "[TEST] {} received broadcast: {}",
                            other_user,
                            response.trim()
                        );
                    }
                }
            }
        }
    }

    // Clean disconnect
    for (username, mut ws_sender, _) in user_connections {
        let leave_msg = ClientMessage::Leave;
        let json = leave_msg.to_json().unwrap();
        ws_sender.send(Message::Text(json)).await.unwrap();
        println!("[TEST] {username} sent leave request");
    }

    println!("=== END DETAILED TRACING ===\n");

    server_handle.abort();
}

#[tokio::test]
async fn test_user_join_leave_flow() {
    let addr = "127.0.0.1:9992";

    // Start server in background
    let server_handle = tokio::spawn(async move {
        run_chat_server(addr).await.unwrap();
    });

    // Give server time to start
    tokio::time::sleep(Duration::from_millis(100)).await;

    println!("\n=== USER JOIN/LEAVE FLOW TEST ===");

    // Test sequential join and leave
    let users = vec!["alice", "bob", "charlie"];

    for username in &users {
        let ws_url = format!("ws://{addr}");
        let (ws_stream, _) = connect_async(ws_url).await.unwrap();
        let (mut ws_sender, mut ws_receiver) = ws_stream.split();

        // Join
        let join_msg = ClientMessage::Join {
            username: username.to_string(),
        };
        let json = join_msg.to_json().unwrap();
        ws_sender.send(Message::Text(json)).await.unwrap();
        println!("[TEST] {username} joining...");

        // Read join response
        if let Ok(Some(Ok(msg))) = timeout(Duration::from_secs(1), ws_receiver.next()).await {
            if let Message::Text(response) = msg {
                let server_msg = ServerMessage::from_json(response.trim()).unwrap();
                match server_msg {
                    ServerMessage::JoinSuccess { user_id } => {
                        println!("[TEST] {username} joined successfully with ID: {user_id}");
                    }
                    _ => panic!("Expected JoinSuccess for {username}"),
                }
            }
        }

        // Send a quick message
        let message = ClientMessage::SendMessage {
            content: format!("Hello from {username}!"),
        };
        let json = message.to_json().unwrap();
        ws_sender.send(Message::Text(json)).await.unwrap();
        println!("[TEST] {username} sent message");

        // Leave immediately
        let leave_msg = ClientMessage::Leave;
        let json = leave_msg.to_json().unwrap();
        ws_sender.send(Message::Text(json)).await.unwrap();
        println!("[TEST] {username} left the chat");

        // Small delay between users
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    println!("=== END JOIN/LEAVE FLOW ===\n");

    server_handle.abort();
}

#[tokio::test]
async fn test_concurrent_message_broadcasting() {
    let addr = "127.0.0.1:9991";

    // Start server in background
    let server_handle = tokio::spawn(async move {
        run_chat_server(addr).await.unwrap();
    });

    // Give server time to start
    tokio::time::sleep(Duration::from_millis(100)).await;

    println!("\n=== CONCURRENT MESSAGE BROADCASTING TEST ===");

    const NUM_USERS: usize = 5;
    let mut user_handles = vec![];

    // Create multiple users that will send messages simultaneously
    for user_id in 0..NUM_USERS {
        let handle = tokio::spawn(async move {
            let ws_url = format!("ws://{addr}");
            if let Ok((ws_stream, _)) = connect_async(ws_url).await {
                let (mut ws_sender, mut ws_receiver) = ws_stream.split();

                let username = format!("user_{user_id}");

                // Join the chat
                let join_msg = ClientMessage::Join {
                    username: username.clone(),
                };
                let json = join_msg.to_json().unwrap();
                ws_sender.send(Message::Text(json)).await.unwrap();
                println!("[USER-{user_id}] Joined the chat");

                // Read join response
                if let Ok(Some(Ok(_))) = timeout(Duration::from_secs(1), ws_receiver.next()).await {
                    // Wait a bit for all users to join
                    tokio::time::sleep(Duration::from_millis(200)).await;

                    // Send multiple messages rapidly
                    for msg_id in 0..3 {
                        let message = ClientMessage::SendMessage {
                            content: format!("Message {msg_id} from {username}"),
                        };
                        let json = message.to_json().unwrap();
                        ws_sender.send(Message::Text(json)).await.unwrap();
                        println!("[USER-{user_id}] Sent message {msg_id}");

                        // Small delay between messages
                        tokio::time::sleep(Duration::from_millis(10)).await;
                    }

                    // Try to read some broadcasts
                    let mut messages_received = 0;
                    for _ in 0..10 {
                        if let Ok(Some(Ok(msg))) =
                            timeout(Duration::from_millis(50), ws_receiver.next()).await
                        {
                            if let Message::Text(response) = msg {
                                if !response.trim().is_empty() && response.contains("Message") {
                                    messages_received += 1;
                                    println!(
                                        "[USER-{}] Received broadcast: {}",
                                        user_id,
                                        response.trim()
                                    );
                                }
                            }
                        }
                    }

                    // Leave
                    let leave_msg = ClientMessage::Leave;
                    let json = leave_msg.to_json().unwrap();
                    ws_sender.send(Message::Text(json)).await.unwrap();
                    println!(
                        "[USER-{user_id}] Left the chat (received {messages_received} messages)"
                    );

                    return messages_received;
                }
            }
            0
        });

        user_handles.push(handle);
    }

    // Wait for all users to complete
    let mut total_received = 0;
    for handle in user_handles {
        if let Ok(received) = handle.await {
            total_received += received;
        }
    }

    println!("Total messages received across all users: {total_received}");
    println!("=== END CONCURRENT BROADCASTING ===\n");

    // Should have received messages (each user sends 3, so with 5 users that's 15 total)
    // Each user should receive messages from others, so total should be > 0
    assert!(total_received > 0, "No messages were received");

    server_handle.abort();
}
