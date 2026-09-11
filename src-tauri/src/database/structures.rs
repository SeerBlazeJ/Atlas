use crate::providers::structures::{ChatMessage, Conversation, Summary};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use surrealdb::types::{RecordId, SurrealValue, ToSql};

#[derive(Serialize, Deserialize, SurrealValue)]
pub struct ConversationDB {
    pub id: Option<RecordId>,
    updated: DateTime<Utc>,
    title: String,
    messages: Vec<ChatMessage>,
}

#[derive(Debug, Serialize, Deserialize, SurrealValue)]
pub struct SummaryDB {
    pub id: RecordId,
    pub title: String,
}

impl From<Conversation> for ConversationDB {
    fn from(value: Conversation) -> Self {
        Self {
            id: value.id.map(|x| RecordId::parse_simple(&x).unwrap()),
            updated: value.updated,
            title: value.title,
            messages: value.messages,
        }
    }
}

impl From<ConversationDB> for Conversation {
    fn from(value: ConversationDB) -> Self {
        Self {
            id: value.id.map(|x| x.to_sql()),
            updated: value.updated,
            title: value.title,
            messages: value.messages,
        }
    }
}

impl From<SummaryDB> for Summary {
    fn from(value: SummaryDB) -> Self {
        Self {
            id: value.id.to_sql(),
            title: value.title,
        }
    }
}
