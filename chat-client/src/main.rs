mod client;

use clap::Parser;
use client::{Args, ChatClient};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    println!(
        "Connecting to chat server at {}:{}...",
        args.host, args.port
    );

    let client = ChatClient::new(args);
    client.run().await?;

    Ok(())
}
