#![allow(clippy::collapsible_match)]

use chat_server::protocol::{ClientMessage, ServerMessage};
use chat_server::websocket::run_chat_server;
use futures_util::{SinkExt, StreamExt};
use std::time::Duration;
use tokio::time::timeout;
use tokio_tungstenite::{connect_async, tungstenite::Message};

#[tokio::test]
async fn test_websocket_connection() {
    let addr = "127.0.0.1:9998";

    // Start server in background
    let server_handle = tokio::spawn(async move {
        run_chat_server(addr).await.unwrap();
    });

    // Give server time to start
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Connect with WebSocket
    let ws_url = format!("ws://{addr}");
    let (ws_stream, _) = connect_async(ws_url).await.unwrap();
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();

    // Join the server
    let join_msg = ClientMessage::Join {
        username: "test_user".to_string(),
    };
    let json = join_msg.to_json().unwrap();
    ws_sender.send(Message::Text(json)).await.unwrap();

    // Read join response
    if let Ok(Some(Ok(msg))) = timeout(Duration::from_secs(2), ws_receiver.next()).await {
        if let Message::Text(response) = msg {
            let server_msg = ServerMessage::from_json(&response).unwrap();
            match server_msg {
                ServerMessage::JoinSuccess { .. } => {
                    println!("Successfully joined chat room");
                }
                _ => panic!("Expected JoinSuccess, got: {server_msg:?}"),
            }
        }
    }

    // Send a message
    let send_msg = ClientMessage::SendMessage {
        content: "Hello WebSocket!".to_string(),
    };
    let json = send_msg.to_json().unwrap();
    ws_sender.send(Message::Text(json)).await.unwrap();

    // Leave the server
    let leave_msg = ClientMessage::Leave;
    let json = leave_msg.to_json().unwrap();
    ws_sender.send(Message::Text(json)).await.unwrap();

    server_handle.abort();
}

#[tokio::test]
async fn test_multiple_websocket_connections() {
    let addr = "127.0.0.1:9997";

    // Start server in background
    let server_handle = tokio::spawn(async move {
        run_chat_server(addr).await.unwrap();
    });

    // Give server time to start
    tokio::time::sleep(Duration::from_millis(100)).await;

    const NUM_CLIENTS: usize = 5;
    let mut handles = vec![];

    for client_id in 0..NUM_CLIENTS {
        let handle = tokio::spawn(async move {
            // Connect with WebSocket
            let ws_url = format!("ws://{addr}");
            if let Ok((ws_stream, _)) = connect_async(ws_url).await {
                let (mut ws_sender, mut ws_receiver) = ws_stream.split();

                // Join the server
                let join_msg = ClientMessage::Join {
                    username: format!("user_{client_id}"),
                };
                let json = join_msg.to_json().unwrap();
                let _ = ws_sender.send(Message::Text(json)).await;

                // Read join response
                if let Ok(Some(Ok(_))) = timeout(Duration::from_secs(2), ws_receiver.next()).await {
                    // Send a message
                    let send_msg = ClientMessage::SendMessage {
                        content: format!("Hello from user {client_id}"),
                    };
                    let json = send_msg.to_json().unwrap();
                    let _ = ws_sender.send(Message::Text(json)).await;

                    // Leave the server
                    let leave_msg = ClientMessage::Leave;
                    let json = leave_msg.to_json().unwrap();
                    let _ = ws_sender.send(Message::Text(json)).await;

                    return true;
                }
            }
            false
        });

        handles.push(handle);
    }

    // Wait for all clients to complete
    let mut successful_connections = 0;
    for handle in handles {
        if let Ok(success) = handle.await {
            if success {
                successful_connections += 1;
            }
        }
    }

    // Assert that most connections were successful
    assert!(
        successful_connections >= NUM_CLIENTS / 2,
        "Only {successful_connections} out of {NUM_CLIENTS} connections successful"
    );

    server_handle.abort();
}
