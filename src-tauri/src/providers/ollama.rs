use std::process::Command;

use crate::providers::structures::*;
use crate::tools::web::WebSearchTool;
use anyhow::{Error, Result};
use futures::StreamExt;
use rig::agent::MultiTurnStreamItem;
use rig::client::{AgentClientExt, Nothing};
use rig::completion::{Message, Prompt};
use rig::providers::ollama;
use rig::streaming::{StreamedAssistantContent, StreamingChat};
use tauri::ipc::Channel;

const DEFAULT_PREAMBLE: &str =
    "You are a helpful virtual assistant, aimed to helping the user in the best way you can, \
     while keeping responses clear, concise and brief.";

pub async fn ollama_prompt_stream(
    model_id: String,
    history: &Vec<ChatMessage>,
    message: &String,
    think: bool,
    channel: Channel<String>,
) -> Result<String, String> {
    let ollama_client =
        ollama::Client::new(Nothing).map_err(|e| format!("Failed to create Ollama client: {e}"))?;

    let agent = ollama_client
        .agent(model_id)
        .default_max_turns(10)
        .tool(WebSearchTool)
        .append_preamble(DEFAULT_PREAMBLE)
        .additional_params(serde_json::json!({ "think": think }))
        .build();

    let history: Vec<Message> = history
        .to_owned()
        .into_iter()
        .map(|x| x.to_rig_message().unwrap())
        .collect();

    let mut stream = agent.stream_chat(&*message, history).await;

    let mut full_text = String::new();

    while let Some(chunk) = stream.next().await {
        match chunk {
            Ok(MultiTurnStreamItem::StreamAssistantItem(StreamedAssistantContent::Text(t))) => {
                full_text.push_str(&t.text);
                channel
                    .send(t.text)
                    .map_err(|e| format!("Failed to send to channel: {e}"))?;
            }
            Ok(_) => {
                // Tool call deltas, user content, CompletionCall, FinalResponse — ignore for plain text
            }
            Err(e) => return Err(format!("Stream error: {e}")),
        }
    }

    Ok(full_text)
}

pub async fn list_ollama_models() -> Vec<String> {
    let output = Command::new("ollama")
        .arg("list")
        .output()
        .expect("Failes to run ollama list command");
    if output.status.success() {
        let output_str = String::from_utf8_lossy(&output.stdout);

        output_str
            .lines()
            .skip(1)
            .filter_map(|line| line.split_whitespace().next().map(|name| name.to_string()))
            .collect()
    } else {
        Vec::new()
    }
}

pub async fn set_chat_name(conversation: String) -> Result<String> {
    ollama::Client::new(Nothing)
        .map_err(|e| Error::msg(format!("Failed summarizing conversation: {e}")))?
        .agent("qwen3.5:0.8b")
        .preamble(
            r#"You are a title generator. Create a short, 3 to 5 word title for this chat based on the user's primary request. 
RULES:
1. Focus ONLY on what the user asked for.
2. Keep it simple and natural (e.g., name the task or topic).
3. Output ONLY the title. No quotes, no periods, no filler words.
EXAMPLES:
Input: "Write a 300 word essay on rust language" -> Output: Rust Language Essay
Input: "How do I center a div in CSS?" -> Output: Centering a CSS Div
Input: "Explain quantum physics to a 5 year old" -> Output: Quantum Physics Explained
    "#,
        )
        .additional_params(serde_json::json!({ "think": false }))
        .build()
        .prompt(conversation)
        .await
        .map_err(|e| Error::msg(format!("Failed summarizing conversation: {e}")))
}
