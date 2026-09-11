use anyhow::{bail, Context, Result};
use rig::client::{EmbeddingsClient, Nothing};
use rig::embeddings::EmbeddingsBuilder;
use rig::providers::ollama;
use rig::vector_store::InsertDocuments;
use rig::Embed;
use rig_surrealdb::SurrealVectorStore;
use serde::{Deserialize, Serialize};
use std::path::Path;
use surrealdb::engine::local::Db;
use surrealdb::Surreal;

/// Local embedding model served by Ollama.
const OLLAMA_EMBED_MODEL: &str = "nomic-embed-text";
const OLLAMA_URL: &str = "http://localhost:11434";

/// Represents an embedded long-term memory chunk or file attachment in SurrealDB.
#[derive(Embed, Serialize, Deserialize, Clone, Debug, Eq, PartialEq, Default)]
pub struct MemoryChunk {
    pub path: String,
    pub metadata: String,
    pub content: String,
    #[serde(skip)]
    #[embed]
    pub embedding_data: String,
}

/// Concrete type of Atlas's SurrealDB-backed vector store using Ollama embeddings.
pub type AtlasVectorStore = SurrealVectorStore<Db, ollama::EmbeddingModel>;

fn ollama_client() -> Result<ollama::Client> {
    ollama::Client::builder()
        .api_key(Nothing)
        .base_url(OLLAMA_URL)
        .build()
        .context("Failed to build Ollama client")
}

pub fn embedding_model() -> Result<ollama::EmbeddingModel> {
    Ok(ollama_client()?.embedding_model(OLLAMA_EMBED_MODEL))
}

/// Builds or re-opens Atlas's RAG vector store on top of an existing SurrealDB handle.
pub fn build_vector_store(db: &Surreal<Db>) -> Result<AtlasVectorStore> {
    Ok(SurrealVectorStore::with_defaults(
        embedding_model()?,
        db.clone(),
    ))
}

/// Saves an individual memory insight or user fact into SurrealDB via Ollama embeddings.
pub async fn save_memory_snippet(
    vector_store: &AtlasVectorStore,
    category: &str,
    content: &str,
    conversation_id: Option<&str>,
) -> Result<()> {
    let model = embedding_model()?;
    let conv_id = conversation_id.unwrap_or("global");
    let timestamp = chrono::Utc::now().to_rfc3339();

    let chunk = MemoryChunk {
        path: format!("memory:{}:{}", conv_id, category),
        metadata: serde_json::json!({
            "timestamp": timestamp,
            "category": category,
            "conversation_id": conv_id,
            "type": "conversation_memory"
        })
        .to_string(),
        content: content.to_string(),
        embedding_data: content.to_string(),
    };

    let embeddings = EmbeddingsBuilder::new(model)
        .document(chunk)?
        .build()
        .await
        .context("Failed to generate embeddings for memory snippet")?;

    vector_store
        .insert_documents(embeddings)
        .await
        .context("Failed to insert memory snippet into SurrealDB")?;

    Ok(())
}

/// Ingests normal text files (.txt, .md, .json, .rs, etc.) into long-term memory.
pub async fn upload_text_file_to_memory(
    vector_store: &AtlasVectorStore,
    file_path: &str,
    conversation_id: Option<&str>,
) -> Result<String> {
    let path = Path::new(file_path);
    if !path.exists() {
        bail!("File does not exist: {}", file_path);
    }

    let raw_text = tokio::fs::read_to_string(path)
        .await
        .with_context(|| format!("Failed to read file {:?}", path))?;

    if raw_text.trim().is_empty() {
        return Ok("File is empty, nothing ingested.".to_string());
    }

    let chunks = chunk_text(&raw_text, 2500, 300);
    let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("file");
    let conv_id = conversation_id.unwrap_or("global");
    let source_tag = format!("file:{}:{}", conv_id, filename);

    let model = embedding_model()?;
    let mut pending = Vec::new();

    for (idx, chunk_str) in chunks.into_iter().enumerate() {
        if chunk_str.trim().is_empty() {
            continue;
        }
        pending.push(MemoryChunk {
            path: source_tag.clone(),
            metadata: serde_json::json!({
                "filename": filename,
                "conversation_id": conv_id,
                "chunk_index": idx,
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "type": "file_attachment"
            })
            .to_string(),
            content: format!("// FILE: {}\n{}", filename, chunk_str),
            embedding_data: chunk_str,
        });
    }

    if pending.is_empty() {
        return Ok("No valid content found to embed.".to_string());
    }

    let indexed_count = pending.len();
    let embeddings = EmbeddingsBuilder::new(model)
        .documents(pending)?
        .build()
        .await
        .context("Failed to generate embeddings for attached file")?;

    vector_store
        .insert_documents(embeddings)
        .await
        .context("Failed to insert file chunks into SurrealDB")?;

    Ok(format!(
        "Successfully attached '{}' ({} chunks) to long-term memory.",
        filename, indexed_count
    ))
}

/// Standard overlapping text chunker.
fn chunk_text(text: &str, chunk_size: usize, overlap: usize) -> Vec<String> {
    let chars: Vec<char> = text.chars().collect();
    let total = chars.len();
    let mut chunks = Vec::new();

    if total == 0 {
        return chunks;
    }

    let mut start = 0;
    while start < total {
        let end = (start + chunk_size).min(total);
        let chunk: String = chars[start..end].iter().collect();
        chunks.push(chunk);

        if end == total {
            break;
        }
        start = if end > overlap { end - overlap } else { end };
    }

    chunks
}
