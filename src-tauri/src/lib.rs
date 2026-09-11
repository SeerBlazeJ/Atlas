#![allow(non_snake_case)]
mod database;
mod providers;
mod tools;
mod voice;

use chrono::Utc;
use database::chat_store::{create_chat_memory, list_chats, update_chat_memory};
use database::document_memory::{build_vector_store, upload_text_file_to_memory};
use providers::ollama::{list_ollama_models, ollama_prompt_stream, set_chat_name};
use providers::openrouter::openrouter_prompt_stream;
use providers::structures::*;
use std::process::Command;
use surrealdb::{
    engine::local::{Db, SurrealKv},
    Surreal,
};
use tauri::ipc::Channel;
use tauri::Manager;
use voice::stt::{send_audio_chunk, start_stream, stop_stream};

/// Used when starting a brand new conversation session.
#[tauri::command]
async fn new_chat(
    message: String,
    on_event: Channel<String>,
    think: bool,
    model_details: (String, ModelType),
    db: tauri::State<'_, Surreal<Db>>,
) -> Result<(String, String), String> {
    let res = run_llm(
        &Vec::new(),
        &message,
        on_event,
        think,
        model_details,
        db.inner().clone(),
        None,
    )
    .await?;

    let history = vec![
        ChatMessage {
            role: Role::User,
            content: message,
        },
        ChatMessage {
            role: Role::Assistant,
            content: res.clone(),
        },
    ];

    let history_str: String = history
        .iter()
        .map(|e| e.to_string())
        .collect::<Vec<String>>()
        .join("\n");

    let summary = set_chat_name(history_str)
        .await
        .map_err(|x| x.to_string())?;

    let id = create_chat_memory(
        &db,
        Conversation {
            id: None,
            updated: Utc::now(),
            title: summary,
            messages: history,
        },
    )
    .await
    .map_err(|x| x.to_string())?;

    Ok((id, res))
}

/// Used when continuing an existing conversation thread.
#[tauri::command]
async fn continue_conversation(
    id: String,
    message: String,
    on_event: Channel<String>,
    think: bool,
    model_details: (String, ModelType),
    db: tauri::State<'_, Surreal<Db>>,
) -> Result<String, String> {
    let history = load_chat_memory(id.clone(), db.clone())
        .await
        .map_err(|x| x.to_string())?;

    let mut messages = history.messages;

    let res = run_llm(
        &messages,
        &message,
        on_event,
        think,
        model_details,
        db.inner().clone(),
        Some(id.clone()),
    )
    .await?;

    messages.push(ChatMessage {
        role: Role::User,
        content: message,
    });
    messages.push(ChatMessage {
        role: Role::Assistant,
        content: res.clone(),
    });

    update_chat_memory(
        &db,
        Conversation {
            id: history.id,
            updated: Utc::now(),
            title: history.title,
            messages,
        },
    )
    .await
    .map_err(|x| x.to_string())?;

    Ok(res)
}

/// Load a single chat's full history from DB by id.
#[tauri::command]
async fn load_chat_memory(
    id: String,
    db: tauri::State<'_, Surreal<Db>>,
) -> Result<Conversation, String> {
    crate::database::chat_store::load_chat_memory(&db, &id)
        .await
        .map_err(|e| format!("Error loading chat: {e}"))
}

/// Ingests a local file into vector memory for contextual RAG retrieval.
#[tauri::command]
async fn upload_file_memory(
    file_path: String,
    conversation_id: Option<String>,
    db: tauri::State<'_, Surreal<Db>>,
) -> Result<String, String> {
    let vector_store =
        build_vector_store(&db).map_err(|e| format!("Failed to build vector store: {e}"))?;

    upload_text_file_to_memory(&vector_store, &file_path, conversation_id.as_deref())
        .await
        .map_err(|e| format!("Error uploading file: {e}"))
}

/// Internal pipeline dispatcher for LLM generation tasks.
async fn run_llm(
    history: &[ChatMessage],
    message: &String,
    on_event: Channel<String>,
    think: bool,
    model_details: (String, ModelType),
    db: Surreal<Db>,
    conversation_id: Option<String>,
) -> Result<String, String> {
    match model_details.1 {
        ModelType::Ollama => {
            ollama_prompt_stream(
                model_details.0,
                history,
                message,
                think,
                db,
                on_event,
                conversation_id,
            )
            .await
        }
        ModelType::OpenRouter => {
            openrouter_prompt_stream(model_details.0, history, think, on_event).await
        }
    }
}

/// Returns a list of conversation summaries containing IDs and titles.
#[tauri::command]
async fn load_chatlist(db: tauri::State<'_, Surreal<Db>>) -> Result<Vec<Summary>, String> {
    list_chats(&db)
        .await
        .map_err(|e| format!("Error loading chats: {e}"))
}

/// Lists all available local and remote models.
#[tauri::command]
async fn list_models() -> Vec<(String, ModelType)> {
    let ls: Vec<(String, ModelType)> = list_ollama_models()
        .await
        .into_iter()
        .map(|x| (x, ModelType::Ollama))
        .collect();
    ls
}

/// Terminate Ollama process before closing application.
fn stop_ollama() {
    #[cfg(target_os = "windows")]
    {
        let _ = Command::new("taskkill")
            .args(["/F", "/IM", "ollama.exe"])
            .output();
    }

    #[cfg(target_os = "linux")]
    {
        let _ = Command::new("systemctl").args(["stop", "ollama"]).output();
        let _ = Command::new("pkill").arg("ollama").output();
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[cfg(target_os = "linux")]
    {
        let _ = Command::new("systemctl").args(["start", "ollama"]).output();
    }
    #[cfg(target_os = "windows")]
    {
        let _ = Command::new("net").args(["start", "ollama"]).output();
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            tauri::async_runtime::block_on(async {
                let db = Surreal::new::<SurrealKv>("AtlasDB")
                    .await
                    .expect("Failed to create DB connection");
                db.use_ns("Atlas")
                    .use_db("main")
                    .await
                    .expect("Failed to select namespace/db");
                let _ = db
                    .query("DEFINE TABLE IF NOT EXISTS documents SCHEMALESS;")
                    .await;
                app.manage(db);
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            new_chat,
            continue_conversation,
            list_models,
            load_chatlist,
            load_chat_memory,
            upload_file_memory,
            send_audio_chunk,
            start_stream,
            stop_stream
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|_app_handle, event| {
            if let tauri::RunEvent::Exit = event {
                stop_ollama();
            }
        });
}
