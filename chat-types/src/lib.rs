use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientMessage {
    Join { username: String },
    Leave,
    SendMessage { content: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerMessage {
    JoinSuccess {
        user_id: Uuid,
    },
    JoinError {
        error: String,
    },
    UserJoined {
        username: String,
    },
    UserLeft {
        username: String,
    },
    Message {
        username: String,
        content: String,
        sender_id: Uuid,
    },
    Error {
        error: String,
    },
}

impl ClientMessage {
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

impl ServerMessage {
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}
