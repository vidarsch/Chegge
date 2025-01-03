use tokio::sync::broadcast;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use sqlx::mysql::MySqlPool;
use sqlx::Error as SqlxError;
use tokio_tungstenite::tungstenite::Message as WsMessage;

#[derive(Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub name: String,
    pub message: String,
}

pub struct ServerState {
    pub tx: broadcast::Sender<ChatMessage>,
    pub pool: MySqlPool,
}

impl ServerState {
    pub fn new(pool: MySqlPool, capacity: usize) -> Arc<Self> {
        let (tx, _rx) = broadcast::channel(capacity);
        Arc::new(ServerState { tx, pool })
    }

    pub async fn broadcast_message(&self, msg: ChatMessage) -> Result<(), broadcast::error::SendError<ChatMessage>> {
        if let Err(e) = self.save_message(&msg).await {
            eprintln!("Error saving message to database: {}", e);
        }
        
        println!("Broadcasting message from {}: {}", msg.name, msg.message);
        self.tx.send(msg).map(|_| ())
    }

    pub async fn broadcast_image(&self, msg: ChatMessage) -> Result<(), broadcast::error::SendError<ChatMessage>> {
        if let Err(e) = self.save_image(&msg).await {
            eprintln!("Error saving message to database: {}", e);
        }
        let mut modified_msg = msg.clone();
        modified_msg.message = format!("img {}", msg.message);
        println!("Broadcasting message from {}: {}", msg.name, modified_msg.message);
        self.tx.send(modified_msg).map(|_| ())
    }

    async fn save_message(&self, msg: &ChatMessage) -> Result<(), SqlxError> {
        sqlx::query(
            "INSERT INTO messages (message, unique_user, created_at) 
             VALUES (?, ?, NOW())"
        )
        .bind(&msg.message)
        .bind(&msg.name)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn save_image(&self, msg: &ChatMessage) -> Result<(), SqlxError> {
        sqlx::query(
            "INSERT INTO messages (img, unique_user, created_at) 
             VALUES (?, ?, NOW())"
        )
        .bind(&msg.message)
        .bind(&msg.name)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn fetch_recent_messages(&self) -> Result<WsMessage, SqlxError> {
        let messages: Vec<(String, String)> = sqlx::query_as(
            "SELECT CONCAT(IF(message IS NULL,'img ','mes '),IFNULL(img,message)), unique_user FROM messages 
             ORDER BY created_at DESC LIMIT 50"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(WsMessage::Text(
            serde_json::to_string(&messages).unwrap()
        ))
    }
}

pub fn format_ws_message(msg: &ChatMessage) -> WsMessage {
    let response = serde_json::json!({
        "type": "message",
        "name": msg.name,
        "message": msg.message
    });
    
    WsMessage::Text(response.to_string())
} 