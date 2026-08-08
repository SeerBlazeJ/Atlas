use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
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

#[derive(Serialize, Deserialize, Clone)]
pub struct ChatMessage {
    pub role: Role,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ModelType {
    OpenRouter,
    Ollama,
}
