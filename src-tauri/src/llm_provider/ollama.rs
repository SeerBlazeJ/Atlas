use std::process::Command;

use futures::StreamExt;
use rig::agent::MultiTurnStreamItem;
use rig::client::{AgentClientExt, Nothing};
use rig::prelude::StreamingPrompt;
use rig::providers::ollama;
use rig::streaming::StreamedAssistantContent;
use serde::{Deserialize, Serialize};
use tauri::ipc::Channel;

#[derive(Serialize, Deserialize, Clone)]
#[allow(non_camel_case_types)]
pub enum Role {
    user,
    system,
    assistant,
}

impl std::fmt::Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Role::user => write!(f, "User"),
            Role::system => write!(f, "System"),
            Role::assistant => write!(f, "Assistant"),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ChatMessage {
    pub role: Role,
    pub content: String,
}

pub async fn prompt_stream(
    model_name: &str,
    messages: Vec<ChatMessage>,
    think: bool,
    channel: Channel<String>,
) -> Result<String, String> {
    let ollama_client =
        ollama::Client::new(Nothing).map_err(|e| format!("Failed to create Ollama client: {e}"))?;

    let agent = ollama_client
        .agent(model_name)
        .additional_params(serde_json::json!({ "think": think }))
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
