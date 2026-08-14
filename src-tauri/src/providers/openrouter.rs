use crate::providers::structures::*;
use dotenvy::dotenv;
use futures::StreamExt;
use rig::prelude::*;
use rig::providers::openrouter;
use rig::streaming::StreamedAssistantContent;
use serde_json::json;
use std::env;
use tauri::ipc::Channel;

// TODO: Fix the inconsistency in model response, till then, this module will go unused.

const TEXT_MODEL_STORE: [(&str, &str); 11] = [
    ("Cohere North mini", "cohere/north-mini-code:free"), // Hybrid
    ("Google Gemma 4", "google/gemma-4-31b-it:free"),     // Hybrid
    ("Google Gemma 4 (A4B)", "google/gemma-4-26b-a4b-it:free"), // Hybrid
    ("InclusionAi", "inclusionai/ling-3.0-tiny:free"),    // hybrid
    (
        "Nvidia Nemotron 3 Nano",
        "nvidia/nemotron-3-nano-30b-a3b:free",
    ), //Hybrid
    (
        "Nvidia Nemotron 3 Nano Omni",
        "nvidia/nemotron-3-nano-omni-30b-a3b-reasoning:free",
    ), //Hybrid
    (
        "Nvidia Nemotron 3 Super",
        "nvidia/nemotron-3-super-120b-a12b:free",
    ), // Hybrid
    (
        "Nvidia Nemotron 3 Ultra",
        "nvidia/nemotron-3-ultra-550b-a55b:free",
    ), //Hybrid
    (
        "Nvidia Nemotron Nano (9B)",
        "nvidia/nemotron-nano-9b-v2:free",
    ), //Hybrid
    ("OpenAI GPT OSS", "openai/gpt-oss-20b:free"),        //Think
    ("Poolside: Laguna XS 2.1", "poolside/laguna-xs-2.1:free"), //Hybrid
];

pub fn get_model_id_binary(target_name: &str) -> Option<&'static str> {
    TEXT_MODEL_STORE
        .binary_search_by_key(&target_name, |&(name, _)| name)
        .ok() // Converts Result<usize, usize> to Option<usize>
        .map(|index| TEXT_MODEL_STORE[index].1) // Gets the ID at that index
}

pub async fn openrouter_prompt_stream(
    model_name: String,
    messages: &Vec<ChatMessage>,
    think: bool,
    channel: Channel<String>,
) -> Result<String, String> {
    dotenv().ok();
    let openrouter_api_key =
        env::var("OPENROUTER_API_KEY").map_err(|_| String::from("Openrouter API key not found"))?;

    let model_id = get_model_id_binary(&model_name).unwrap_or("google/gemma-4-26b-a4b-it:free");

    let openrouter_client = openrouter::Client::new(openrouter_api_key)
        .map_err(|_| String::from("Failed ot build openrouter client"))?;

    let agent = openrouter_client
        .agent(model_id)
        .additional_params(json!({
            "reasoning": {
                "enabled": think
            }
        }))
        .build();

    let prompt = messages
        .iter()
        .map(|m| format!("{}: {}", m.role, m.content))
        .collect::<Vec<String>>()
        .join("\n\n");

    let mut stream = agent.stream_prompt(&prompt).await;
    let mut full_text = String::new();

    while let Some(chunk) = stream.next().await {
        match chunk {
            Ok(MultiTurnStreamItem::StreamAssistantItem(StreamedAssistantContent::Text(t))) => {
                full_text.push_str(&t.text);
                channel
                    .send(t.text)
                    .map_err(|e| format!("Failed to send top channel: {e}"))?;
            }
            Ok(_) => {
                // Tool call/user content/completion call/final response
            }
            Err(e) => return Err(format!("Streaming Error: {e}")),
        }
    }
    Ok(full_text)
}

pub fn list_openrouter_models() -> Vec<String> {
    TEXT_MODEL_STORE
        .into_iter()
        .map(|x| x.0.to_string())
        .collect()
}
