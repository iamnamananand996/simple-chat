mod messages;
mod users;
mod websocket;

use clap::Parser;
use websocket::run_chat_server;

#[derive(Parser, Debug)]
#[command(author, version, about = "Chat Server", long_about = None)]
struct Args {
    #[arg(short, long, default_value = "127.0.0.1:8080")]
    addr: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    run_chat_server(&args.addr).await?;
    Ok(())
}
