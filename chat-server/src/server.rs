use chat_types::{ClientMessage, ServerMessage};
use uuid::Uuid;
use dashmap::DashMap;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast;

#[derive(Debug, Clone)]
pub struct User {
    pub username: String,
}

pub struct ChatServer {
    users: Arc<DashMap<Uuid, User>>,
    usernames: Arc<DashMap<String, Uuid>>,
    broadcast_tx: broadcast::Sender<ServerMessage>,
}

impl Default for ChatServer {
    fn default() -> Self {
        Self::new()
    }
}

impl ChatServer {
    pub fn new() -> Self {
        // Increased capacity for high throughput
        let (broadcast_tx, _) = broadcast::channel(10000);

        Self {
            users: Arc::new(DashMap::new()),
            usernames: Arc::new(DashMap::new()),
            broadcast_tx,
        }
    }

    pub async fn run(&self, addr: &str) -> Result<(), Box<dyn std::error::Error>> {
        let listener = TcpListener::bind(addr).await?;
        println!("Chat server listening on {addr}");

        loop {
            let (stream, addr) = listener.accept().await?;
            println!("New connection from {addr}");

            let server = self.clone();
            tokio::spawn(async move {
                if let Err(e) = server.handle_client(stream).await {
                    eprintln!("Error handling client: {e}");
                }
            });
        }
    }

    async fn handle_client(&self, stream: TcpStream) -> Result<(), Box<dyn std::error::Error>> {
        let (reader, writer) = stream.into_split();
        let mut reader = BufReader::new(reader);
        let mut line = String::new();
        let mut user_id: Option<Uuid> = None;
        let mut broadcast_rx = self.broadcast_tx.subscribe();

        let writer = Arc::new(tokio::sync::Mutex::new(writer));
        let writer_clone = Arc::clone(&writer);

        tokio::spawn(async move {
            while let Ok(msg) = broadcast_rx.recv().await {
                // Move JSON serialization to thread pool to avoid blocking
                match tokio::task::spawn_blocking(move || msg.to_json()).await {
                    Ok(Ok(json)) => {
                        let mut writer = writer_clone.lock().await;
                        if writer
                            .write_all(format!("{json}\n").as_bytes())
                            .await
                            .is_err()
                        {
                            break;
                        }
                    }
                    _ => break, // Error in serialization or spawn_blocking
                }
            }
        });

        loop {
            line.clear();
            match reader.read_line(&mut line).await {
                Ok(0) => break, // Connection closed
                Ok(_) => {
                    let line = line.trim();
                    if line.is_empty() {
                        continue;
                    }

                    // Move JSON parsing to thread pool
                    let line_owned = line.to_string();
                    match tokio::task::spawn_blocking(move || ClientMessage::from_json(&line_owned)).await {
                        Ok(Ok(msg)) => match self.process_message(msg, &mut user_id).await {
                            Ok(Some(response)) => {
                                // Move JSON serialization to thread pool
                                match tokio::task::spawn_blocking(move || response.to_json()).await {
                                    Ok(Ok(json)) => {
                                        let mut writer = writer.lock().await;
                                        writer.write_all(format!("{json}\n").as_bytes()).await?;
                                    }
                                    _ => {} // Error in serialization
                                }
                            }
                            Ok(None) => {}
                            Err(e) => {
                                let error_msg = ServerMessage::Error {
                                    error: e.to_string(),
                                };
                                match tokio::task::spawn_blocking(move || error_msg.to_json()).await {
                                    Ok(Ok(json)) => {
                                        let mut writer = writer.lock().await;
                                        writer.write_all(format!("{json}\n").as_bytes()).await?;
                                    }
                                    _ => {} // Error in serialization
                                }
                            }
                        },
                        Ok(Err(e)) => {
                            let error_msg = ServerMessage::Error {
                                error: format!("Invalid message format: {e}"),
                            };
                            match tokio::task::spawn_blocking(move || error_msg.to_json()).await {
                                Ok(Ok(json)) => {
                                    let mut writer = writer.lock().await;
                                    writer.write_all(format!("{json}\n").as_bytes()).await?;
                                }
                                _ => {} // Error in serialization
                            }
                        },
                        Err(_) => {
                            let error_msg = ServerMessage::Error {
                                error: "Failed to parse message".to_string(),
                            };
                            match tokio::task::spawn_blocking(move || error_msg.to_json()).await {
                                Ok(Ok(json)) => {
                                    let mut writer = writer.lock().await;
                                    writer.write_all(format!("{json}\n").as_bytes()).await?;
                                }
                                _ => {} // Error in serialization
                            }
                        }
                    }
                }
                Err(_) => break,
            }
        }

        // Cleanup on disconnect
        if let Some(id) = user_id {
            self.remove_user(id).await;
        }

        Ok(())
    }

    async fn process_message(
        &self,
        msg: ClientMessage,
        user_id: &mut Option<Uuid>,
    ) -> Result<Option<ServerMessage>, Box<dyn std::error::Error + Send + Sync>> {
        match msg {
            ClientMessage::Join { username } => {
                if user_id.is_some() {
                    return Ok(Some(ServerMessage::Error {
                        error: "Already joined".to_string(),
                    }));
                }

                let id = self.add_user(username.clone()).await?;
                *user_id = Some(id);

                println!("[JOIN] User '{}' ({}) joined the chat", username, id);

                // Broadcast to others that user joined
                let join_msg = ServerMessage::UserJoined {
                    username: username.clone(),
                };
                let user_count = self.users.len();
                println!("[BROADCAST] Broadcasting join notification for '{}' to {} users", username, user_count);
                let _ = self.broadcast_tx.send(join_msg);

                Ok(Some(ServerMessage::JoinSuccess { user_id: id }))
            }
            ClientMessage::Leave => {
                if let Some(id) = user_id.take() {
                    if let Some(user) = self.users.get(&id) {
                        let username = user.username.clone();
                        println!("[LEAVE] User '{}' ({}) left the chat", username, id);
                    }
                    self.remove_user(id).await;
                }
                Ok(None)
            }
            ClientMessage::SendMessage { content } => {
                if let Some(id) = user_id {
                    // DashMap provides lock-free read access
                    if let Some(user) = self.users.get(id) {
                        let username = user.username.clone();
                        println!("[MESSAGE] User '{}' ({}) sent: '{}'", username, id, content);
                        
                        let msg = ServerMessage::Message {
                            username: username.clone(),
                            content: content.clone(),
                        };
                        
                        // Log broadcast to all connected users
                        let user_count = self.users.len();
                        println!("[BROADCAST] Broadcasting message from '{}' to {} connected users", username, user_count);
                        
                        let _ = self.broadcast_tx.send(msg);
                    }
                }
                Ok(None)
            }
        }
    }

    async fn add_user(
        &self,
        username: String,
    ) -> Result<Uuid, Box<dyn std::error::Error + Send + Sync>> {
        // DashMap provides atomic operations - no locks needed!
        if self.usernames.contains_key(&username) {
            return Err("Username already taken".into());
        }

        let id = Uuid::new_v4();
        let user = User {
            username: username.clone(),
        };

        // Insert atomically - if username was taken between check and insert,
        // this will return the existing value
        match self.usernames.insert(username, id) {
            Some(_) => {
                // Username was taken between our check and insert
                Err("Username already taken".into())
            }
            None => {
                // Successfully inserted, now add the user
                self.users.insert(id, user);
                Ok(id)
            }
        }
    }

    async fn remove_user(&self, user_id: Uuid) {
        // DashMap provides atomic operations - no locks needed!
        if let Some((_, user)) = self.users.remove(&user_id) {
            self.usernames.remove(&user.username);

            let remaining_users = self.users.len();
            println!("[BROADCAST] Broadcasting leave notification for '{}' to {} remaining users", user.username, remaining_users);

            // Broadcast that user left
            let leave_msg = ServerMessage::UserLeft {
                username: user.username,
            };
            let _ = self.broadcast_tx.send(leave_msg);
        }
    }
}

impl Clone for ChatServer {
    fn clone(&self) -> Self {
        Self {
            users: Arc::clone(&self.users),
            usernames: Arc::clone(&self.usernames),
            broadcast_tx: self.broadcast_tx.clone(),
        }
    }
}
