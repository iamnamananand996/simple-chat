use crate::messages::handle_message;
use crate::users::{create_user_stores, UserStore, UsernameStore};
use chat_types::{ClientMessage, ServerMessage};
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast;
use tokio_tungstenite::{accept_async, tungstenite::Message};
use uuid::Uuid;

pub async fn run_chat_server(addr: &str) -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind(addr).await?;
    println!("Chat server listening on {addr} (WebSocket)");

    let (broadcast_tx, _) = broadcast::channel::<ServerMessage>(10000);
    let (users, usernames) = create_user_stores();

    loop {
        let (stream, addr) = listener.accept().await?;
        println!("New WebSocket connection from {addr}");

        let users_clone = Arc::clone(&users);
        let usernames_clone = Arc::clone(&usernames);
        let broadcast_tx_clone = broadcast_tx.clone();

        tokio::spawn(async move {
            if let Err(e) =
                handle_websocket_client(stream, users_clone, usernames_clone, broadcast_tx_clone)
                    .await
            {
                eprintln!("Error handling WebSocket client: {e}");
            }
        });
    }
}

async fn handle_websocket_client(
    stream: TcpStream,
    users: UserStore,
    usernames: UsernameStore,
    broadcast_tx: broadcast::Sender<ServerMessage>,
) -> Result<(), Box<dyn std::error::Error>> {
    let websocket = accept_async(stream).await?;
    let (ws_sender, ws_receiver) = websocket.split();

    let mut user_id: Option<Uuid> = None;
    let broadcast_rx = broadcast_tx.subscribe();

    // Use channels to coordinate between the two tasks
    let (outgoing_tx, mut outgoing_rx) = tokio::sync::mpsc::unbounded_channel::<ServerMessage>();

    // Spawn task to handle outgoing messages (both responses and broadcasts)
    let mut ws_sender = ws_sender;
    let outgoing_task = tokio::spawn(async move {
        loop {
            tokio::select! {
                msg = outgoing_rx.recv() => {
                    match msg {
                        Some(server_msg) => {
                            if let Ok(json) = server_msg.to_json() {
                                if ws_sender.send(Message::Text(json)).await.is_err() {
                                    break;
                                }
                            }
                        }
                        None => break, // Channel closed
                    }
                }
            }
        }
    });

    // Spawn task to handle broadcast messages
    let broadcast_sender = outgoing_tx.clone();
    let mut broadcast_rx = broadcast_rx;
    tokio::spawn(async move {
        while let Ok(server_msg) = broadcast_rx.recv().await {
            if broadcast_sender.send(server_msg).is_err() {
                break; // Receiver dropped
            }
        }
    });

    // Handle incoming WebSocket messages
    let mut ws_receiver = ws_receiver;
    let users_clone = Arc::clone(&users);
    let usernames_clone = Arc::clone(&usernames);
    let broadcast_tx_clone = broadcast_tx.clone();

    let incoming_task = tokio::spawn(async move {
        while let Some(msg) = ws_receiver.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    if let Ok(client_msg) = ClientMessage::from_json(&text) {
                        if let Some(response) = handle_message(
                            &users_clone,
                            &usernames_clone,
                            &broadcast_tx_clone,
                            user_id,
                            client_msg,
                        )
                        .await
                        {
                            // Handle special case for join success to store user_id
                            if let ServerMessage::JoinSuccess { user_id: id } = &response {
                                user_id = Some(*id);
                            }

                            let _ = outgoing_tx.send(response);
                        }
                    } else {
                        let error_msg = ServerMessage::Error {
                            error: "Invalid message format".to_string(),
                        };
                        let _ = outgoing_tx.send(error_msg);
                    }
                }
                Ok(Message::Close(_)) => break,
                Err(_) => break,
                _ => {} // Ignore other message types
            }
        }

        // Cleanup on disconnect
        if user_id.is_some() {
            let _ = handle_message(
                &users_clone,
                &usernames_clone,
                &broadcast_tx_clone,
                user_id,
                ClientMessage::Leave,
            )
            .await;
        }
    });

    // Wait for either task to complete
    tokio::select! {
        _ = incoming_task => {},
        _ = outgoing_task => {},
    }

    Ok(())
}
