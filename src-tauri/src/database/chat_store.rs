use super::structures::{ConversationDB, SummaryDB};
use crate::providers::structures::{Conversation, Summary};
use anyhow::Error;
use surrealdb::{
    engine::local::SurrealKv,
    types::{RecordId, ToSql},
    Surreal,
};

pub async fn create_chat_memory(data: Conversation) -> Result<String, Error> {
    let db = Surreal::new::<SurrealKv>("AtlasDB").await?;
    db.use_ns("Atlas").use_db("Conversations").await?;
    let messages: ConversationDB = ConversationDB::from(data);
    let new_conv: ConversationDB = db.create("chats").content(messages).await?.unwrap();
    let id = new_conv.id.map(|x| x.to_sql()).unwrap();
    Ok(id)
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

// pub async fn append_to_conv_mem

pub async fn load_chat_memory(id: &String) -> Result<Conversation, Error> {
    let db = Surreal::new::<SurrealKv>("AtlasDB").await?;
    db.use_ns("Atlas").use_db("Conversations").await?;
    let rid: RecordId = RecordId::parse_simple(&id)?;
    let res: Option<ConversationDB> = db.select(rid).await?;
    if res.is_none() {
        return Err(Error::msg("No chat found for the given ID"));
    }
    let res = res.unwrap();
    Ok(Conversation::from(res))
}

pub async fn list_chats() -> Result<Vec<Summary>, Error> {
    let db = Surreal::new::<SurrealKv>("AtlasDB").await?;
    db.use_ns("Atlas").use_db("Conversations").await?;
    let mut res = db
        .query(
            "SELECT id, title FROM (
    SELECT id, title, created FROM chats ORDER BY created DESC
)",
        )
        .await?;
    let chats: Vec<SummaryDB> = res.take(0)?;
    Ok(chats.into_iter().map(Summary::from).collect())
}
