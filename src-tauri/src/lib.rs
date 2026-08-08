#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod chat_completion;
use chat_completion::chat_structures::*;
use chat_completion::{ollama::ollama_prompt_stream, openrouter::openrouter_prompt_stream};
use tauri::ipc::Channel;

use crate::chat_completion::ollama::list_ollama_models;
use crate::chat_completion::openrouter::list_openrouter_models;

//  TODO: history of chat functionality is not properly implemented yet - awaiting DB connections
/// Call the LLM takes in the following params:
///
/// `Message`: message sent by the user
///
/// `on_event`: A channel where response can be live streamed as tokens are generated
///
/// `think`: Boolean value to enable/disable reasoning
///
/// `model_details`: A tuple of String that is the model ID and the name of the provider - Ollama/OpenRouter
#[tauri::command]
async fn run_llm(
    message: String,
    on_event: Channel<String>,
    think: bool,
    model_details: (String, ModelType),
) -> Result<String, String> {
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
    match model_details.1 {
        ModelType::Ollama => ollama_prompt_stream(model_details.0, history, think, on_event).await,
        ModelType::OpenRouter => {
            openrouter_prompt_stream(model_details.0, history, think, on_event).await
        }
    }
}

/// List all the available models, returns the values as a vector of ("ModelName",Ollama/OpenRouter)
#[tauri::command]
async fn list_models() -> Vec<(String, ModelType)> {
    let mut ls: Vec<(String, ModelType)> = list_ollama_models()
        .await
        .into_iter()
        .map(|x| (x, ModelType::Ollama))
        .collect();
    let x: () = list_openrouter_models()
        .into_iter()
        .map(|x| ls.push((x, ModelType::OpenRouter)))
        .collect();
    ls
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
