use crate::providers::structures::*;
use dotenvy::dotenv;
use futures::StreamExt;
use rig::prelude::*;
use rig::providers::openrouter;
use rig::streaming::StreamedAssistantContent;
use serde_json::json;
use std::env;
use tauri::ipc::Channel;

const TEXT_MODEL_STORE: [(&str, &str); 13] = [
    ("InclusionAi", "inclusionai/ling-3.0-tiny:free"),
    ("Poolside: Laguna S 2.1", "poolside/laguna-s-2.1:free"),
    ("Poolside: Laguna XS 2.1", "poolside/laguna-xs-2.1:free"),
    ("Cohere North mini", "cohere/north-mini-code:free"),
    (
        "Nvidia Nemotron 3.5 (content safety)",
        "nvidia/nemotron-3.5-content-safety:free",
    ),
    (
        "Nvidia Nemotron 3 Ultra",
        "nvidia/nemotron-3-ultra-550b-a55b:free",
    ),
    (
        "Nvidia Nemotron 3 Nano",
        "nvidia/nemotron-3-nano-30b-a3b:free",
    ),
    (
        "Nvidia Nemotron Nano (9B)",
        "nvidia/nemotron-nano-9b-v2:free",
    ),
    (
        "Nvidia Nemotron 3 Nano Omni",
        "nvidia/nemotron-3-nano-omni-30b-a3b-reasoning:free",
    ),
    (
        "Nvidia Nemotron 3 Super",
        "nvidia/nemotron-3-super-120b-a12b:free",
    ),
    ("Google Gemma 4", "google/gemma-4-31b-it:free"),
    ("Google Gemma 4 (A4B)", "google/gemma-4-26b-a4b-it:free"),
    ("OpenAI GPT OSS", "openai/gpt-oss-20b:free"),
];

pub async fn openrouter_prompt_stream(
    model_id: String,
    messages: Vec<ChatMessage>,
    think: bool,
    channel: Channel<String>,
) -> Result<String, String> {
    dotenv().ok();
    let openrouter_api_key =
        env::var("OPENROUTER_API_KEY").map_err(|_| String::from("Openrouter API key not found"))?;

    // let openrouter_api_key =
    //     String::from("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");

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
