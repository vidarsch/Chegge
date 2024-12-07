mod chat;
use chat::{ServerState, ChatMessage, format_ws_message};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite;
use futures::{SinkExt, StreamExt};
use serde::Deserialize;
use std::net::SocketAddr;
use tokio_tungstenite::tungstenite::handshake::server::{Request, Response};
use sqlx::mysql::MySqlPool;
use sqlx::Error as SqlxError;
use std::sync::Arc;

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
        username: "root".to_string(),
        password: "u92kxaCU".to_string(),
        database: "chegge_man".to_string(),
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

    let state = ServerState::new(pool);
    
    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
    println!("WebSocket server listening on ws://127.0.0.1:8080");

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
    // Get connection details before upgrading to WebSocket
    let callback = |request: &Request, response: Response| {
        // Extract headers we're interested in
        let user_agent = request.headers()
            .get("user-agent")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("Unknown")
            .to_string();
        
        // Instead of awaiting here, we'll just log the info
        println!("New connection from:");
        println!("  IP Address: {}", addr);
        println!("  User Agent: {}", user_agent);
        println!("  Headers:");
        for (name, value) in request.headers() {
            if let Ok(value_str) = value.to_str() {
                println!("    {}: {}", name, value_str);
            }
        }

        // Spawn a task to handle the database insertion
        let state = state.clone();
        let addr = addr.clone();
        let user_agent = user_agent.clone();
        tokio::spawn(async move {
            if let Err(e) = query_users(&state.pool, addr, user_agent).await {
                eprintln!("Error logging user: {}", e);
            }
        });

        Ok(response)
    };

    let ws_stream = tokio_tungstenite::accept_hdr_async(
        stream,
        callback
    ).await.expect("Failed to accept WebSocket connection");

    println!("WebSocket connection established");

    let (mut write, mut read) = ws_stream.split();
    let mut rx = state.tx.subscribe();

    // Create a channel for sending messages from the read task to the write task
    let (tx_internal, mut rx_internal) = tokio::sync::mpsc::channel(32);

    // Handle broadcast messages and internal messages
    let write_task = tokio::spawn(async move {
        loop {
            tokio::select! {
                // Handle broadcast messages
                result = rx.recv() => {
                    match result {
                        Ok(msg) => {
                            if let Err(e) = write.send(format_ws_message(&msg)).await {
                                println!("Error sending broadcast message: {}", e);
                                break;
                            }
                        }
                        Err(e) => {
                            println!("Broadcast channel error: {}", e);
                            break;
                        }
                    }
                }
                // Handle internal messages (like fetch_messages responses)
                Some(msg) = rx_internal.recv() => {
                    if let Err(e) = write.send(msg).await {
                        println!("Error sending internal message: {}", e);
                        break;
                    }
                }
                else => break,
            }
        }
    });

    // Handle incoming messages
    let state_clone = state.clone();
    let tx_internal_clone = tx_internal.clone();
    let read_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = read.next().await {
            if let Ok(text) = msg.to_text() {
                if let Ok(incoming) = serde_json::from_str::<IncomingMessage>(text) {
                    match incoming.r#type.as_str() {
                        "message" => {
                            let chat_msg = ChatMessage {
                                name: incoming.name.unwrap_or_else(|| "Anonymous".to_string()),
                                message: incoming.message.unwrap_or_default(),
                            };
                            
                            if let Err(e) = state_clone.broadcast_message(chat_msg).await {
                                eprintln!("Error broadcasting message: {}", e);
                            }
                        },
                        "fetch_messages" => {
                            if let Ok(messages) = state_clone.fetch_recent_messages().await {
                                if let Err(e) = tx_internal_clone.send(messages).await {
                                    eprintln!("Error sending message history: {}", e);
                                }
                            }
                        },
                        // ... handle other message types ...
                        _ => println!("Unknown message type"),
                    }
                }
            }
        }
    });

    tokio::select! {
        _ = write_task => println!("Write task completed"),
        _ = read_task => println!("Read task completed"),
    }

    println!("Connection closed");
}
async fn get_terrain(height: u32, width: u32) {
    println!("Height: {}, Width: {}", height, width);
}

async fn query_terrain(pool: &MySqlPool, height: u32, width: u32) -> Result<tokio_tungstenite::tungstenite::Message, SqlxError> {
    // Example query - adjust according to your schema
    let row: (String,) = sqlx::query_as(
        "SELECT terrain_data FROM terrains WHERE height = ? AND width = ?"
    )
    .bind(height)
    .bind(width)
    .fetch_one(pool)
    .await?;

    Ok(tokio_tungstenite::tungstenite::Message::Text(row.0))
}
async fn query_users(
    pool: &MySqlPool, 
    addr: SocketAddr,
    accept_language: String,
) -> Result<(), SqlxError> {
    // Insert user connection data
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

async fn update_message(pool: &MySqlPool, message: String, name: String) -> Result<(), SqlxError> {
    let name = if name.is_empty() { "Anonymous".to_string() } else { name };
        sqlx::query(
            "INSERT INTO messages (message, unique_user, created_at) 
         VALUES (?, ?, NOW())"
    )
    .bind(message)
    .bind(name)
    .execute(pool)
    .await?;

    Ok(())
}
async fn update_name(pool: &MySqlPool, message: String) -> Result<(), SqlxError> {
    sqlx::query(
        "INSERT INTO messages (message, unique_users, created_at) 
         VALUES (?, UUID(), NOW())"
    )
    .bind(message)
    .execute(pool)
    .await?;

    Ok(())
}

async fn fetch_messages(pool: &MySqlPool) -> Result<tokio_tungstenite::tungstenite::Message, SqlxError> {
    let messages: Vec<(String, String)> = sqlx::query_as(
        "SELECT message, unique_user FROM messages ORDER BY created_at DESC LIMIT 50"
    ).fetch_all(pool).await?;

    Ok(tokio_tungstenite::tungstenite::Message::Text(
        serde_json::to_string(&messages).unwrap()
    ))
}
