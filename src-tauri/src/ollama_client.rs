use futures_util::TryStreamExt;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tauri::ipc::Channel;
use tokio::io::AsyncBufReadExt;
use tokio_util::io::StreamReader;

pub const OLLAMA_URL: &str = "http://localhost:11434";

#[derive(Serialize, Deserialize, Clone)]
#[allow(non_camel_case_types)]
pub enum Role {
    user,
    system,
    assistant,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ChatMessage {
    pub role: Role,
    pub content: String,
}

#[derive(Deserialize)]
struct OllamaStreamChunk {
    message: Option<OllamaMessagePart>,
    done: bool,
}

#[derive(Deserialize)]
struct OllamaMessagePart {
    content: String,
}

/// Streams tokens from Ollama's /api/chat endpoint using StreamReader + lines(),
/// forwarding each decoded chunk to the frontend via a Tauri Channel as it arrives.
pub async fn prompt_stream(
    model: &str,
    messages: Vec<ChatMessage>,
    think: bool,
    channel: Channel<String>,
) -> Result<String, String> {
    let http_client = reqwest::Client::new();

    let payload = json!({
        "model": model,
        "messages": messages,
        "stream": true,
        "options": {
            "temperature": 0.35,
            "repeat_penalty": 1.5
        },
        "think": think
    });

    let response = http_client
        .post(format!("{}/api/chat", OLLAMA_URL))
        .json(&payload)
        .send()
        .await
        .map_err(|e| format!("Connection to Ollama failed: {e}"))?;

    // reqwest's error type must be converted to std::io::Error for StreamReader, since it expects a Stream<Item = Result<Bytes, std::io::Error>>.
    let byte_stream = response
        .bytes_stream()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e));

    let stream_reader = StreamReader::new(byte_stream);
    let mut lines = stream_reader.lines();

    let mut full_text = String::new();

    while let Some(line) = lines
        .next_line()
        .await
        .map_err(|e| format!("Stream read error: {e}"))?
    {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let chunk: OllamaStreamChunk =
            serde_json::from_str(trimmed).map_err(|e| format!("Failed to parse chunk: {e}"))?;

        if let Some(part) = chunk.message {
            full_text.push_str(&part.content);
            channel
                .send(part.content)
                .map_err(|e| format!("Failed to send to channel: {e}"))?;
        }

        if chunk.done {
            break;
        }
    }

    Ok(full_text)
}
