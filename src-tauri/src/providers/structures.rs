use serde::{Deserialize, Serialize};
use surrealdb::types::SurrealValue;

#[derive(Clone, Serialize, Deserialize, SurrealValue)]
#[allow(non_camel_case_types)]
pub enum Role {
    user,
    system,
    assistant,
}

impl std::fmt::Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Role::user => write!(f, "User"),
            Role::system => write!(f, "System"),
            Role::assistant => write!(f, "Assistant"),
        }
    }
}

#[derive(Clone, Serialize, Deserialize, SurrealValue)]
pub struct ChatMessage {
    pub role: Role,
    pub content: String,
}

#[derive(Serialize, Deserialize)]
pub enum ModelType {
    OpenRouter,
    Ollama,
}

#[derive(Serialize, Deserialize)]
pub struct Conversation {
    pub id: Option<String>,
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
