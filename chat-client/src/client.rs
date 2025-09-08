use chat_types::{ClientMessage, ServerMessage};
use clap::Parser;
use std::io::{self, Write};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;

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

pub struct ChatClient {
    args: Args,
}

impl ChatClient {
    pub fn new(args: Args) -> Self {
        Self { args }
    }

    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let addr = format!("{}:{}", self.args.host, self.args.port);
        let stream = TcpStream::connect(&addr).await?;
        println!("Connected to chat server at {addr}");

        let (reader, mut writer) = stream.into_split();
        let reader = BufReader::new(reader);

        // Send join message
        let join_msg = ClientMessage::Join {
            username: self.args.username.clone(),
        };
        let json = join_msg.to_json()?;
        writer.write_all(format!("{json}\n").as_bytes()).await?;

        // Spawn task to handle incoming messages
        let mut server_reader = reader;
        let client_username = self.args.username.clone();
        tokio::spawn(async move {
            let mut line = String::new();
            loop {
                line.clear();
                match server_reader.read_line(&mut line).await {
                    Ok(0) => {
                        println!("Server disconnected");
                        break;
                    }
                    Ok(_) => {
                        let line = line.trim();
                        if line.is_empty() {
                            continue;
                        }

                        match ServerMessage::from_json(line) {
                            Ok(msg) => match msg {
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
                            },
                            Err(e) => {
                                println!("Failed to parse server message: {e}");
                            }
                        }
                    }
                    Err(e) => {
                        println!("Error reading from server: {e}");
                        break;
                    }
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
                writer.write_all(format!("{json}\n").as_bytes()).await?;
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
                writer.write_all(format!("{json}\n").as_bytes()).await?;
            } else {
                println!(
                    "Unknown command. Use 'send <message>' to send a message or 'leave' to exit"
                );
            }
        }
        Ok(())
    }
}
