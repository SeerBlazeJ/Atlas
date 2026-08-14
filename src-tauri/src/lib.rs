#![allow(non_snake_case)]
mod providers;
use std::process::Command;

use anyhow::Result;
use providers::structures::*;
use providers::{ollama::ollama_prompt_stream, openrouter::openrouter_prompt_stream};
use tauri::ipc::Channel;
mod database;
use crate::database::chat_store::{
    create_chat_memory, list_chats, load_chat_memory, update_chat_memory,
};
use crate::providers::ollama::{list_ollama_models, set_chat_name};
// use crate::providers::openrouter::list_openrouter_models;

#[tauri::command]
async fn new_chat(
    message: String,
    on_event: Channel<String>,
    think: bool,
    model_details: (String, ModelType),
) -> Result<(String, String), String> {
    let mut history = vec![ChatMessage {
        role: Role::user,
        content: message,
    }];
    let res = run_llm(&history, on_event, think, model_details).await?;
    history.push(ChatMessage {
        role: Role::assistant,
        content: res.clone(),
    });
    let history_str: String = history
        .iter()
        .map(|e| e.to_string())
        .collect::<Vec<String>>()
        .join("\n");
    let summary = set_chat_name(history_str)
        .await
        .map_err(|x| x.to_string())?;
    let id = create_chat_memory(Conversation {
        id: None,
        title: summary,
        messages: history,
    })
    .await
    .map_err(|x| x.to_string())?;
    Ok((id, res))
}

#[tauri::command]
async fn continue_conversation(
    id: String,
    message: String,
    on_event: Channel<String>,
    think: bool,
    model_details: (String, ModelType),
) -> Result<String, String> {
    let history = load_chat_memory(&id).await.map_err(|x| x.to_string())?;
    let mut messages = history.messages;
    messages.push(ChatMessage {
        role: Role::user,
        content: message,
    });
    let res = run_llm(&messages, on_event, think, model_details).await?;
    messages.push(ChatMessage {
        role: Role::assistant,
        content: res.clone(),
    });
    let _ = update_chat_memory(Conversation {
        id: history.id,
        title: history.title,
        messages,
    })
    .await
    .map_err(|x| x.to_string())?;
    Ok(res)
}

/// Call the LLM takes in the following params:
///
/// `History`: History of the conversation till now
///
/// `on_event`: A channel where response can be live streamed as tokens are generated
///
/// `think`: Boolean value to enable/disable reasoning
///
/// `model_details`: A tuple of String that is the model ID and the name of the provider - Ollama/OpenRouter (Suspended Temporarily)
async fn run_llm(
    history: &Vec<ChatMessage>,
    on_event: Channel<String>,
    think: bool,
    model_details: (String, ModelType),
) -> Result<String, String> {
    // Load history and add it to context using .add_context, pass it as a param to the function
    match model_details.1 {
        ModelType::Ollama => ollama_prompt_stream(model_details.0, history, think, on_event).await,
        ModelType::OpenRouter => {
            openrouter_prompt_stream(model_details.0, history, think, on_event).await
        }
    }
}

#[tauri::command]
async fn load_chatlist() -> Result<Vec<Summary>, String> {
    list_chats()
        .await
        .map_err(|e| format!("Error loading chats: {e}"))
}

/// List all the available models, returns the values as a vector of ("ModelName",Ollama/OpenRouter)
#[tauri::command]
async fn list_models() -> Vec<(String, ModelType)> {
    let ls: Vec<(String, ModelType)> = list_ollama_models()
        .await
        .into_iter()
        .map(|x| (x, ModelType::Ollama))
        .collect();
    // let _: () = list_openrouter_models()
    //     .into_iter()
    //     .map(|x| ls.push((x, ModelType::OpenRouter)))
    //     .collect();
    ls
}

fn stop_ollama() {
    #[cfg(target_os = "windows")]
    {
        // Windows: Force kill the ollama.exe process
        let _ = Command::new("taskkill")
            .args(["/F", "/IM", "ollama.exe"])
            .output();
    }

    #[cfg(target_os = "linux")]
    {
        // Linux: Attempt to stop the systemd service
        // let _ = Command::new("systemctl").args(["stop", "ollama"]).output();

        // Linux Fallback: kill the process if it was started manually via terminal
        let _ = Command::new("pkill").arg("ollama").output();
    }
}

// Run function - runs the main tauri app
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[cfg(target_os = "linux")]
    {
        let _ = Command::new("systemctl").args(["start", "ollama"]).output();
    }
    #[cfg(target_os = "windows")]
    {
        let _ = Command::new("net");
    }
    .args(["start", "ollama"])
    .output();
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            new_chat,
            continue_conversation,
            list_models,
            load_chatlist
        ])
        .build(tauri::generate_context!()) // Use .build() instead of .run() directly
        .expect("error while building tauri application")
        .run(|_app_handle, event| {
            // Listen for the Exit event and stop the Ollama service
            if let tauri::RunEvent::Exit = event {
                stop_ollama();
            }
        });
}
