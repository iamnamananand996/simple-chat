mod server;

use clap::Parser;
use server::ChatServer;

#[derive(Parser, Debug)]
#[command(author, version, about = "Chat Server", long_about = None)]
struct Args {
    #[arg(short, long, default_value = "127.0.0.1:8080")]
    addr: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    
    println!("Starting chat server");

    let server = ChatServer::new();
    server.run(&args.addr).await?;
    
    Ok(())
}
