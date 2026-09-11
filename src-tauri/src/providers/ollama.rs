use crate::database::document_memory::{build_vector_store, save_memory_snippet};
use crate::providers::structures::*;
use crate::tools::remember::RememberTool;
use crate::tools::*;
use crate::voice::tts::VoiceModule;
use anyhow::{Error, Result};
use futures::StreamExt;
use rig::agent::MultiTurnStreamItem;
use rig::client::{AgentClientExt, Nothing};
use rig::completion::{Message, Prompt};
use rig::providers::ollama;
use rig::streaming::{StreamedAssistantContent, StreamingChat};
use std::process::Command;
use surrealdb::engine::local::Db;
use surrealdb::Surreal;
use tauri::ipc::Channel;

const DEFAULT_PREAMBLE: &str = r#"You are Atlas, a sophisticated, highly capable AI assistant managing the user's Linux system.

Persona and Tone:
- Address the user politely and formally as "sir".
- Emulate the demeanor of J.A.R.V.I.S.: calm, composed, articulate, highly efficient, with subtle dry wit.
- Deliver speech naturally, fluently, and concisely.

TTS & Voice Formatting Rules (STRICT):
- All output is streamed directly to a Text-to-Speech voice engine.
- NEVER use Markdown formatting: no asterisks, bolding, italics, bullet points, headers, backticks, or code blocks.
- NEVER use emojis, ASCII diagrams, tables, or raw URLs.
- Write in clean, complete spoken English sentences with standard punctuation (periods, commas, question marks).
- Spell out or speak technical symbols naturally rather than printing them (for example, say "slash dev slash sda" instead of "/dev/sda", or "percent" instead of "%").
- Express numbers and versions in plain conversational phrases that sound natural when read aloud.

Capabilities and Tool Use:
You inspect and manage the system using your available tools:
- searching files and context (atlas_find, project_context),
- web search (web_search), firewall management (firewall_manager),
- explicitly remembering key information into long-term memory (remember_information).

Operational Guidelines:
1. Always prefer a read-only action first (inspect, status, check) before proposing changes.
2. System actions that modify files or state require confirmation. Clearly ask for authorization first and state precisely what will be changed.
3. Summarize tool findings concisely in conversational speech; mention only the essential metrics or status."#;

/// Uses a lightweight background Rig Agent turn to analyze if a conversation contains persistent insights worth saving.
pub async fn evaluate_and_persist_insights(
    ollama_client: &ollama::Client,
    user_prompt: &str,
    assistant_response: &str,
    db: Surreal<Db>,
    conversation_id: &str,
) {
    let eval_prompt = format!(
        "Analyze this user-assistant conversation turn.\n\
        Determine if the user shared any permanent personal facts, preferences, project specifics, or explicit instructions worth remembering for future sessions.\n\
        Ignore generic questions, standard coding assistance, or temporary queries.\n\n\
        If there are NO permanent facts or preferences, reply ONLY with 'NONE'.\n\
        If there ARE worth-saving insights, output each insight as a bullet point starting with '- '.\n\n\
        User: {}\n\
        Assistant: {}",
        user_prompt, assistant_response
    );

    // Build a lightweight Rig agent for evaluation
    let eval_agent = ollama_client
        .agent("qwen3.5:0.8b")
        .preamble("You are a conversation memory extraction module. Extract only persistent user facts, preferences, or project details. Output 'NONE' if no long-term facts exist.")
        .additional_params(serde_json::json!({ "think": false }))
        .build();

    if let Ok(response) = eval_agent.prompt(eval_prompt).await {
        let trimmed_resp = response.trim();
        if !trimmed_resp.to_uppercase().contains("NONE") {
            if let Ok(vector_store) = build_vector_store(&db) {
                for line in trimmed_resp.lines() {
                    let line_trimmed = line.trim();
                    if line_trimmed.starts_with('-') || line_trimmed.starts_with('*') {
                        let fact = line_trimmed.trim_start_matches(&['-', '*', ' '][..]).trim();
                        if !fact.is_empty() {
                            let _ = save_memory_snippet(
                                &vector_store,
                                "auto_extracted_insight",
                                fact,
                                Some(conversation_id),
                            )
                            .await;
                        }
                    }
                }
            }
        }
    }
}

pub async fn ollama_prompt_stream(
    model_id: String,
    history: &[ChatMessage],
    message: &String,
    think: bool,
    db: Surreal<Db>,
    channel: Channel<String>,
    conversation_id: Option<String>,
) -> Result<String, String> {
    let ollama_client =
        ollama::Client::new(Nothing).map_err(|e| format!("Failed to create Ollama client: {e}"))?;

    let vector_store_rag =
        build_vector_store(&db).map_err(|e| format!("Failed to build vector store: {e}"))?;
    let vector_store_remember =
        build_vector_store(&db).map_err(|e| format!("Failed to build vector store: {e}"))?;

    // Construct the Rig Agent with all tools and vector context attached
    let agent = ollama_client
        .agent(&model_id)
        .default_max_turns(20)
        .tool(web::WebSearchTool)
        .tool(firewall::FirewallTool)
        .tool(RememberTool {
            vector_store: vector_store_remember,
            conversation_id: conversation_id.clone(),
        })
        .dynamic_context(5, vector_store_rag)
        .append_preamble(DEFAULT_PREAMBLE)
        .additional_params(serde_json::json!({ "think": think }))
        .build();

    let history: Vec<Message> = history
        .iter()
        .map(|x| x.to_rig_message().unwrap())
        .collect();

    let mut voice_module = VoiceModule::new().unwrap_or_else(|e| {
        eprintln!("Voice module failed to start: {}", e);
        panic!("TTS Error")
    });

    let mut stream = agent.stream_chat(message, history).await;
    let mut full_text = String::new();

    while let Some(chunk) = stream.next().await {
        match chunk {
            Ok(MultiTurnStreamItem::StreamAssistantItem(StreamedAssistantContent::Text(t))) => {
                full_text.push_str(&t.text);
                voice_module.push_text(&t.text);
                channel
                    .send(t.text)
                    .map_err(|e| format!("Failed to send to channel: {e}"))?;
            }
            Ok(MultiTurnStreamItem::StreamAssistantItem(StreamedAssistantContent::ToolCall {
                tool_call,
                ..
            })) => {
                let line = format!("\n> ⚙️ {}\n", tool_call.function.name);
                full_text.push_str(&line);
            }
            Ok(MultiTurnStreamItem::ToolExecutionCommitted { tool_call, .. }) => {
                let _ = tool_call.function.name;
            }
            Ok(_) => {
                // Ignore internal deltas
            }
            Err(e) => return Err(format!("Stream error: {e}")),
        }
    }

    voice_module.flush();

    // Trigger post-conversation insight persistence asynchronously using a secondary Rig Agent call
    if let Some(conv_id) = conversation_id {
        let client_clone = ollama_client.clone();
        let prompt_clone = message.clone();
        let resp_clone = full_text.clone();
        let db_clone = db.clone();

        tokio::spawn(async move {
            evaluate_and_persist_insights(
                &client_clone,
                &prompt_clone,
                &resp_clone,
                db_clone,
                &conv_id,
            )
            .await;
        });
    }

    Ok(full_text)
}

pub async fn list_ollama_models() -> Vec<String> {
    let output = Command::new("ollama")
        .arg("list")
        .output()
        .expect("Failed to run ollama list command");
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
