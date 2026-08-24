use chrono::{DateTime, Utc};
use rig::{
    completion::Message,
    message::{AssistantContent, UserContent},
    OneOrMany,
};
use serde::{Deserialize, Serialize};
use surrealdb::types::SurrealValue;

#[derive(Clone, Copy, Serialize, Deserialize, SurrealValue)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    User,
    System,
    Assistant,
}

impl std::fmt::Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Role::User => write!(f, "User"),
            Role::System => write!(f, "System"),
            Role::Assistant => write!(f, "Assistant"),
        }
    }
}

#[derive(Clone, Serialize, Deserialize, SurrealValue)]
pub struct ChatMessage {
    pub role: Role,
    pub content: String,
}

impl ChatMessage {
    pub fn to_rig_message(&self) -> Option<Message> {
        match self.role {
            Role::User => Some(Message::User {
                content: OneOrMany::one(UserContent::text(self.content.clone())),
            }),
            Role::Assistant => Some(Message::Assistant {
                content: OneOrMany::one(AssistantContent::text(self.content.clone())),
                id: None,
            }),
            Role::System => None, // handled separately via preamble
        }
    }
}

#[derive(Serialize, Deserialize)]
pub enum ModelType {
    OpenRouter,
    Ollama,
}

#[derive(Serialize, Deserialize)]
pub struct Conversation {
    pub id: Option<String>,
    pub created: DateTime<Utc>,
    pub title: String,
    pub messages: Vec<ChatMessage>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Summary {
    pub id: String,
    pub title: String,
}

impl ToString for ChatMessage {
    fn to_string(&self) -> String {
        format!("{} : {}", self.role, self.content)
    }
}
