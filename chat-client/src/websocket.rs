use chat_types::{ClientMessage, ServerMessage};
use clap::Parser;
use futures_util::{SinkExt, StreamExt};
use std::io::{self, Write};
use tokio_tungstenite::{connect_async, tungstenite::Message};

#[derive(Parser, Debug, Clone)]
#[command(author, version, about = "Chat Client", long_about = None)]
pub struct Args {
    #[arg(long, default_value = "127.0.0.1")]
    pub host: String,

    #[arg(short, long, default_value = "8080")]
    pub port: u16,

    #[arg(short, long)]
    pub username: String,
}

pub async fn run_chat_client(args: Args) -> Result<(), Box<dyn std::error::Error>> {
    let ws_url = format!("ws://{}:{}", args.host, args.port);
    let (ws_stream, _) = connect_async(&ws_url).await?;
    println!("Connected to WebSocket chat server at {ws_url}");

    let (mut ws_sender, mut ws_receiver) = ws_stream.split();

    // Send join message
    let join_msg = ClientMessage::Join {
        username: args.username.clone(),
    };
    let json = join_msg.to_json()?;
    ws_sender.send(Message::Text(json)).await?;

    // Spawn task to handle incoming WebSocket messages
    let client_username = args.username.clone();
    tokio::spawn(async move {
        while let Some(msg) = ws_receiver.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    match ServerMessage::from_json(&text) {
                        Ok(msg) => {
                            match msg {
                                ServerMessage::JoinSuccess { .. } => {
                                    println!("Successfully joined the chat room!");
                                    println!("Commands: 'send <message>' to send a message, 'leave' to exit");
                                }
                                ServerMessage::JoinError { error } => {
                                    println!("Failed to join: {error}");
                                    return;
                                }
                                ServerMessage::UserJoined { username } => {
                                    // Don't show our own join notification
                                    if username != client_username {
                                        println!("* {username} joined the chat");
                                    }
                                }
                                ServerMessage::UserLeft { username } => {
                                    // Don't show our own leave notification
                                    if username != client_username {
                                        println!("* {username} left the chat");
                                    }
                                }
                                ServerMessage::Message { username, content } => {
                                    println!("{username}: {content}");
                                }
                                ServerMessage::Error { error } => {
                                    println!("Error: {error}");
                                }
                            }
                        }
                        Err(e) => {
                            println!("Failed to parse server message: {e}");
                        }
                    }
                }
                Ok(Message::Close(_)) => {
                    println!("Server disconnected");
                    break;
                }
                Err(e) => {
                    println!("WebSocket error: {e}");
                    break;
                }
                _ => {} // Ignore other message types
            }
        }
    });

    // Handle user input
    loop {
        print!("> ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if input.is_empty() {
            continue;
        }

        if input == "leave" {
            let leave_msg = ClientMessage::Leave;
            let json = leave_msg.to_json()?;
            ws_sender.send(Message::Text(json)).await?;
            println!("Goodbye!");
            break;
        } else if let Some(message) = input.strip_prefix("send ") {
            if message.is_empty() {
                println!("Please provide a message to send");
                continue;
            }

            let send_msg = ClientMessage::SendMessage {
                content: message.to_string(),
            };
            let json = send_msg.to_json()?;
            ws_sender.send(Message::Text(json)).await?;
        } else {
            println!("Unknown command. Use 'send <message>' to send a message or 'leave' to exit");
        }
    }
    Ok(())
}
