use chat_server::protocol::ClientMessage;
use chat_server::websocket::run_chat_server;
use criterion::{criterion_group, criterion_main, Criterion};
use futures_util::{SinkExt, StreamExt};
use std::time::Duration;
use tokio::time::timeout;
use tokio_tungstenite::{connect_async, tungstenite::Message};

async fn setup_server(addr: &str) -> tokio::task::JoinHandle<()> {
    let addr = addr.to_string();

    let server_handle = tokio::spawn(async move {
        run_chat_server(&addr).await.unwrap();
    });

    // Give server time to start
    tokio::time::sleep(Duration::from_millis(100)).await;

    server_handle
}

async fn send_messages_benchmark(num_clients: usize, messages_per_client: usize) {
    let addr = "127.0.0.1:19999";
    let server_handle = setup_server(addr).await;

    // Spawn multiple clients concurrently
    let mut handles = vec![];

    for client_id in 0..num_clients {
        let handle = tokio::spawn(async move {
            let ws_url = format!("ws://{addr}");
            if let Ok((ws_stream, _)) = connect_async(ws_url).await {
                let (mut ws_sender, mut ws_receiver) = ws_stream.split();

                // Join the server
                let join_msg = ClientMessage::Join {
                    username: format!("bench_user_{client_id}"),
                };
                let json = join_msg.to_json().unwrap();
                let _ = ws_sender.send(Message::Text(json)).await;

                // Read join response
                if let Ok(Some(Ok(_))) = timeout(Duration::from_secs(2), ws_receiver.next()).await {
                    // Send messages rapidly
                    for msg_id in 0..messages_per_client {
                        let message = ClientMessage::SendMessage {
                            content: format!("Bench message {msg_id} from user {client_id}"),
                        };
                        let json = message.to_json().unwrap();
                        let _ = ws_sender.send(Message::Text(json)).await;
                    }

                    // Leave the server
                    let leave_msg = ClientMessage::Leave;
                    let json = leave_msg.to_json().unwrap();
                    let _ = ws_sender.send(Message::Text(json)).await;
                }
            }
        });

        handles.push(handle);
    }

    // Wait for all clients to complete
    for handle in handles {
        handle.await.unwrap();
    }

    server_handle.abort();
}

fn throughput_benchmark(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let mut group = c.benchmark_group("chat_server_throughput");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(20));

    // Benchmark different client/message combinations
    for &(clients, messages) in &[(10, 10), (50, 20), (100, 10)] {
        let total_messages = clients * messages;

        group.bench_function(format!("{clients}_clients_{messages}_msgs_each"), |b| {
            b.iter(|| rt.block_on(async { send_messages_benchmark(clients, messages).await }));
        });

        group.throughput(criterion::Throughput::Elements(total_messages as u64));
    }

    group.finish();
}

fn latency_benchmark(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("message_latency", |b| {
        b.iter(|| {
            rt.block_on(async {
                let addr = "127.0.0.1:19998";
                let server_handle = setup_server(addr).await;

                // Connect two clients with WebSocket
                let ws_url = format!("ws://{addr}");

                let (ws_stream1, _) = connect_async(&ws_url).await.unwrap();
                let (mut ws_sender1, mut ws_receiver1) = ws_stream1.split();

                let (ws_stream2, _) = connect_async(&ws_url).await.unwrap();
                let (mut ws_sender2, mut ws_receiver2) = ws_stream2.split();

                // Join both clients
                for (ws_sender, username) in
                    [(&mut ws_sender1, "sender"), (&mut ws_sender2, "receiver")]
                {
                    let join_msg = ClientMessage::Join {
                        username: username.to_string(),
                    };
                    let json = join_msg.to_json().unwrap();
                    ws_sender.send(Message::Text(json)).await.unwrap();
                }

                // Read join responses
                for ws_receiver in [&mut ws_receiver1, &mut ws_receiver2] {
                    timeout(Duration::from_secs(1), ws_receiver.next())
                        .await
                        .unwrap();
                }

                // Measure message latency
                let message = ClientMessage::SendMessage {
                    content: "Latency test message".to_string(),
                };
                let json = message.to_json().unwrap();

                let _start = std::time::Instant::now();
                ws_sender1.send(Message::Text(json)).await.unwrap();

                // Read the broadcast on receiver
                ws_receiver2.next().await.unwrap();

                server_handle.abort();
            })
        });
    });
}

criterion_group!(benches, throughput_benchmark, latency_benchmark);
criterion_main!(benches);
