use super::structures::ConversationDB;
use crate::providers::structures::Conversation;
use anyhow::Error;
use surrealdb::{engine::local::SurrealKv, Surreal};

pub async fn create_chat_memory(data: Conversation) -> Result<(), Error> {
    let db = Surreal::new::<SurrealKv>("AtlasDB").await?;
    db.use_ns("Atlas").use_db("Conversations").await?;
    let messages: ConversationDB = ConversationDB::from(data);
    let _: Option<ConversationDB> = db.create("chats").content(messages).await?;
    Ok(())
}
pub async fn update_chat_memory(data: Conversation) -> Result<(), Error> {
    let db = Surreal::new::<SurrealKv>("AtlasDB").await?;
    db.use_ns("Atlas").use_db("Conversations").await?;
    let messages: ConversationDB = ConversationDB::from(data);
    let id = messages
        .id
        .as_ref()
        .ok_or(Error::msg("Could not resolve RecordID"))?;
    let _: Option<ConversationDB> = db.update(id).content(messages).await?;
    Ok(())
}
pub async fn load_chat_memory() -> Result<(), Error> {
    todo!()
}
pub async fn list_chats() -> Result<Conversation, Error> {
    todo!()
}
