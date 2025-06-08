pub mod poll;
pub mod user;

use serde::{Deserialize, Serialize};

// Common structures and enums
#[derive(Debug, Serialize, Deserialize)]
pub struct FlashMessage {
    pub message_type: MessageType,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageType {
    Success,
    Info,
    Warning,
    Error,
}

impl ToString for MessageType {
    fn to_string(&self) -> String {
        match self {
            MessageType::Success => "success".to_string(),
            MessageType::Info => "info".to_string(),
            MessageType::Warning => "warning".to_string(),
            MessageType::Error => "error".to_string(),
        }
    }
}
