mod websocket;

use clap::Parser;
use websocket::{run_chat_client, Args};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    println!(
        "Connecting to chat server at {}:{}...",
        args.host, args.port
    );
    run_chat_client(args).await?;
    Ok(())
}
