#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod providers;
use providers::structures::*;
use providers::{ollama::ollama_prompt_stream, openrouter::openrouter_prompt_stream};
use tauri::ipc::Channel;
mod database;
use crate::providers::ollama::list_ollama_models;
use crate::providers::openrouter::list_openrouter_models;

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
            id: None,
            role: Role::system,
            content: "You are a helpful virtual assistant, aimed to helping the user in the best way you can, while keeping responses clear, concise and brief.".to_string(),
        },
        ChatMessage {
            id: None,
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

// Run function - runs the main tauri app
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![run_llm, list_models])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
