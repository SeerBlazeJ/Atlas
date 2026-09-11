use super::structures::{ConversationDB, SummaryDB};
use crate::providers::structures::{Conversation, Summary};
use anyhow::Error;
use surrealdb::{
    engine::local::Db,
    types::{RecordId, ToSql},
    Surreal,
};

pub async fn create_chat_memory(db: &Surreal<Db>, data: Conversation) -> Result<String, Error> {
    let messages: ConversationDB = ConversationDB::from(data);
    let new_conv: ConversationDB = db.create("chats").content(messages).await?.unwrap();
    let id = new_conv.id.map(|x| x.to_sql()).unwrap();
    Ok(id)
}

pub async fn update_chat_memory(db: &Surreal<Db>, data: Conversation) -> Result<(), Error> {
    let messages: ConversationDB = ConversationDB::from(data);
    let id = messages
        .id
        .as_ref()
        .ok_or(Error::msg("Could not resolve RecordID"))?;
    let _: Option<ConversationDB> = db.update(id).content(messages).await?;
    Ok(())
}

// pub async fn append_to_conv_mem

pub async fn load_chat_memory(db: &Surreal<Db>, id: &str) -> Result<Conversation, Error> {
    let rid: RecordId = RecordId::parse_simple(id)?;
    let res: Option<ConversationDB> = db.select(rid).await?;
    if res.is_none() {
        return Err(Error::msg("No chat found for the given ID"));
    }
    let res = res.unwrap();
    Ok(Conversation::from(res))
}

pub async fn list_chats(db: &Surreal<Db>) -> Result<Vec<Summary>, Error> {
    let mut res = db
        .query(
            "SELECT id, title FROM (
    SELECT id, title, updated FROM chats ORDER BY updated DESC
)",
        )
        .await?;
    let chats: Vec<SummaryDB> = res.take(0)?;
    Ok(chats.into_iter().map(Summary::from).collect())
}
