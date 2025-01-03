use tokio::sync::broadcast;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use sqlx::mysql::MySqlPool;
use sqlx::Error as SqlxError;
use tokio_tungstenite::tungstenite::Message as WsMessage;
use base64::{decode, DecodeError};
use log::{error, info};
use anyhow::{Result, anyhow};
use std::error::Error;

#[derive(Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub name: String,
    pub message: Option<String>,
    pub image: Option<String>, // Base64-encoded image string
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

    pub async fn broadcast_message(&self, msg: ChatMessage) -> Result<()> {
        if let Err(e) = self.save_message(&msg).await {
            eprintln!("Error saving message to database: {}", e);

        }

        println!("Broadcasting message from {}: {}", msg.name, 
            msg.message.as_deref().unwrap_or("<no message>"));
        
        self.tx.send(msg).map_err(|e| {
            error!("Broadcast send error: {}", e);
            anyhow!("Broadcast send error")
        })?;

        Ok(())
    }

    pub async fn broadcast_image(&self, msg: ChatMessage) -> Result<()> {
        // Decode the Base64 image
        let decoded_image = match decode(&msg.image.clone().unwrap_or_default()) {
            Ok(bytes) => bytes,
            Err(e) => {
                error!("Failed to decode Base64 image: {}", e);
                return Err(anyhow!("Failed to decode Base64 image: {}", e));
            }
        };

        if let Err(e) = self.save_image(&msg.name, &decoded_image).await {
            error!("Error saving image to database: {}", e);
        }

        let modified_msg = ChatMessage {
            name: msg.name.clone(),
            message: None, 
            image: Some(msg.image.unwrap_or_default()), 
        };
        info!("Broadcasting image from {}.", msg.name);
        if let Err(e) = self.tx.send(modified_msg) {
            error!("Broadcast send error: {}", e);
            return Err(anyhow!("Broadcast send error: {}", e).into());
        }

        Ok(())
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

    async fn save_image(&self, username: &str, image_data: &[u8]) -> Result<(), SqlxError> {
        sqlx::query(
            "INSERT INTO messages (img, unique_user, created_at) 
             VALUES (?, ?, NOW())"
        )
        .bind(image_data)
        .bind(username)
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
    let msg_type = if msg.image.is_some() {
        "message-image"
    } else {
        "message"
    };
    
    let response = serde_json::json!({
        "type": msg_type,
        "name": msg.name,
        "message": msg.message,
        "image": msg.image,
    });
    
    WsMessage::Text(response.to_string())
} 