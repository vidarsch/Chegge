mod chat;
use chat::{ServerState, ChatMessage, format_ws_message};
use tokio::sync::broadcast;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite;
use futures::{SinkExt, StreamExt};
use serde::Deserialize;
use std::net::SocketAddr;
use tokio_tungstenite::tungstenite::handshake::server::{Request, Response};
use sqlx::mysql::MySqlPool;
use sqlx::Error as SqlxError;
use std::sync::Arc;
use tokio::sync::broadcast::error::RecvError;
use tokio::time::{interval, Duration};
use std::sync::atomic::{AtomicBool, Ordering};
use tokio_tungstenite::tungstenite::Message as WsMessage;

#[derive(Deserialize)]
struct IncomingMessage {
    r#type: String,
    height: Option<u32>,
    width: Option<u32>,
    message: Option<String>,
    name: Option<String>,
}

struct DbConfig {
    host: String,
    port: u16,
    username: String,
    password: String,
    database: String,
}

struct ClientInfo {
    addr: SocketAddr,
    user_agent: String,
    headers: Vec<(String, String)>,
}

#[tokio::main]
async fn main() {
    let db_config = DbConfig {
        host: "localhost".to_string(),
        port: 3306,
        username: "admin".to_string(),
        password: "mNJw246a5CkL".to_string(),
        database: "cheggeserver".to_string(),
    };

    let db_url = format!(
        "mysql://{}:{}@{}:{}/{}",
        db_config.username,
        db_config.password,
        db_config.host,
        db_config.port,
        db_config.database
    );

    let pool = MySqlPool::connect(&db_url)
        .await
        .expect("Failed to connect to database");

    let state = ServerState::new(pool, 1000);
    
    let listener = TcpListener::bind("92.113.145.13:8080").await.unwrap();
    println!("WebSocket server listening on ws://92.113.145.13:8080");

    while let Ok((stream, addr)) = listener.accept().await {
        let state = state.clone();
        tokio::spawn(handle_connection(stream, addr, state));
    }
}

async fn handle_connection(
    stream: TcpStream, 
    addr: SocketAddr, 
    state: Arc<ServerState>
) {
    let callback = |request: &Request, response: Response| {
        let user_agent = request.headers()
            .get("user-agent")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("Unknown")
            .to_string();
        
        println!("New connection from {}", addr);
        println!("User Agent: {}", user_agent);
        for (name, value) in request.headers() {
            if let Ok(value_str) = value.to_str() {
                println!("  {}: {}", name, value_str);
            }
        }

        let state_clone = state.clone();
        let addr_clone = addr.clone();
        let user_agent_clone = user_agent.clone();
        tokio::spawn(async move {
            if let Err(e) = query_users(&state_clone.pool, addr_clone, user_agent_clone).await {
                eprintln!("Error logging user: {}", e);
            }
        });

        Ok(response)
    };

    let ws_stream = match tokio_tungstenite::accept_hdr_async(stream, callback).await {
        Ok(ws) => ws,
        Err(e) => {
            eprintln!("WebSocket handshake failed for {}: {}", addr, e);
            return;
        }
    };

    println!("WebSocket connection established with {}", addr);

    let (mut write, mut read) = ws_stream.split();
    let mut rx = state.tx.subscribe();

    let (tx_internal, mut rx_internal) = tokio::sync::mpsc::channel(32);

    let is_active = Arc::new(AtomicBool::new(true));
    let is_active_write = is_active.clone();
    let is_active_read = is_active.clone();

    let mut ping_interval = interval(Duration::from_secs(30));

    let mut write_task = Some(tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = ping_interval.tick() => {
                    if let Err(e) = write.send(WsMessage::Ping(vec![])).await {
                        eprintln!("Failed to send ping to {}: {}", addr, e);
                        break;
                    }
                }

                result = rx.recv() => {
                    match result {
                        Ok(msg) => {
                            let formatted_message = format_ws_message(&msg);
                            println!("Sending message to {}: {:?}", addr, formatted_message);
                            if let Err(e) = write.send(formatted_message).await {
                                eprintln!("Error sending message to {}: {}", addr, e);
                                is_active_write.store(false, Ordering::SeqCst);
                                break;
                            }
                        }
                        Err(RecvError::Lagged(count)) => {
                            eprintln!("{} lagged by {} messages.", addr, count);
                            continue;
                        }
                        Err(e) => {
                            eprintln!("Broadcast error for {}: {}", addr, e);
                            is_active_write.store(false, Ordering::SeqCst);
                            break;
                        }
                    }
                }

                Some(msg) = rx_internal.recv() => {
                    println!("Sending internal message to {}: {:?}", addr, msg);
                    if let Err(e) = write.send(msg).await {
                        eprintln!("Error sending internal message to {}: {}", addr, e);
                        is_active_write.store(false, Ordering::SeqCst);
                        break;
                    }
                }

                else => break,
            }
        }
    }));

    let mut read_task = Some(tokio::spawn({
        let state_clone = state.clone();
        let tx_internal_clone = tx_internal.clone();
        let is_active_read = is_active_read.clone();
        async move {
            while let Some(result) = read.next().await {
                match result {
                    Ok(msg) => {
                        if let Ok(text) = msg.to_text() {
                            if let Ok(incoming) = serde_json::from_str::<IncomingMessage>(text) {
                                match incoming.r#type.as_str() {
                                    "message" => {
                                        let chat_msg = ChatMessage {
                                            name: incoming.name.unwrap_or_else(|| "Anonymous".to_string()),
                                            message: incoming.message.unwrap_or_default(),
                                        };
                                        println!("Received message from {}: {}", addr, chat_msg.message);
                                        if let Err(e) = state_clone.broadcast_message(chat_msg).await {
                                            eprintln!("Error broadcasting message: {}", e);
                                        }
                                    },
                                    "fetch_messages" => {
                                        if let Ok(messages) = state_clone.fetch_recent_messages().await {
                                            println!("Sending message history to {}", addr);
                                            if let Err(e) = tx_internal_clone.send(messages).await {
                                                eprintln!("Error sending message history to {}: {}", addr, e);
                                            }
                                        }
                                    },
                                    _ => {
                                        eprintln!("Unknown message type from {}: {}", addr, incoming.r#type);
                                    }
                                }
                            } else {
                                eprintln!("Failed to parse incoming message from {}: {}", addr, text);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Error receiving message from {}: {}", addr, e);
                        is_active_read.store(false, Ordering::SeqCst);
                        break;
                    }
                }
            }
            is_active_read.store(false, Ordering::SeqCst);
        }
    }));

    tokio::select! {
        _ = write_task.as_mut().unwrap() => {
            println!("Write task completed for {}", addr);
            if let Some(read) = read_task.take() {
                read.abort();
            }
        },
        _ = read_task.as_mut().unwrap() => {
            println!("Read task completed for {}", addr);
            if let Some(write) = write_task.take() {
                write.abort();
            }
        },
    }

    println!("Connection with {} closed", addr);
}

async fn get_terrain(height: u32, width: u32) {
    println!("Height: {}, Width: {}", height, width);
}

async fn query_users(
    pool: &MySqlPool, 
    addr: SocketAddr,
    accept_language: String,
) -> Result<(), SqlxError> {
    sqlx::query(
        "INSERT INTO users_log (ip_address, accept_language, created_at) 
         VALUES (?, ?, NOW())"
    )
    .bind(addr.to_string())
    .bind(accept_language)
    .execute(pool)
    .await?;

    Ok(())
}
