#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod llm_provider;
use llm_provider::ollama::{prompt_stream, ChatMessage, Role};
use tauri::ipc::Channel;

use crate::llm_provider::ollama::list_ollama_models;

// Can be called from the frontend, Interface used to chat with the LLMs
//  TODO: history of chat functionality is not properly implemented yet - awaiting DB connections
#[tauri::command]
async fn run_llm(message: String, on_event: Channel<String>) -> Result<String, String> {
    let history = vec![
        ChatMessage {
            role: Role::system,
            content: "You are a helpful virtual assistant, aimed to helping the user in the best way you can, while keeping responses clear, concise and brief.".to_string(),
        },
        ChatMessage {
            role: Role::user,
            content: message,
        },
    ];

    prompt_stream("qwen3.5:0.8b", history, false, on_event).await
}

/// List all the available models
#[tauri::command]
async fn list_models() -> Vec<String> {
    list_ollama_models().await
}

// Run function - runs the main tauri app
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![run_llm])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
