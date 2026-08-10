use crate::providers::structures::{ChatMessage, Conversation};
use serde::{Deserialize, Serialize};
use surrealdb::types::{RecordId, SurrealValue, ToSql};

#[derive(Serialize, Deserialize, SurrealValue)]
pub struct ConversationDB {
    pub id: Option<RecordId>,
    messages: Vec<ChatMessage>,
}

impl From<Conversation> for ConversationDB {
    fn from(value: Conversation) -> Self {
        Self {
            id: value
                .id
                .and_then(|x| Some(RecordId::parse_simple(&x).unwrap())),
            messages: value.messages,
        }
    }
}

impl From<ConversationDB> for Conversation {
    fn from(value: ConversationDB) -> Self {
        Self {
            id: value.id.map(|x| x.to_sql()),
            messages: value.messages,
        }
    }
}
