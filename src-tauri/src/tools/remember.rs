use crate::database::document_memory::{save_memory_snippet, AtlasVectorStore};
use rig::tool::{Tool, ToolContext};
use serde::Deserialize;
use serde_json::Value;
use std::fmt;

#[derive(Deserialize)]
pub struct RememberArgs {
    pub fact: String,
    pub category: Option<String>,
}

#[derive(Debug)]
pub struct StringError(pub String);
impl fmt::Display for StringError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for StringError {}

pub struct RememberTool {
    pub vector_store: AtlasVectorStore,
    pub conversation_id: Option<String>,
}

impl Tool for RememberTool {
    const NAME: &'static str = "remember_information";
    type Args = RememberArgs;
    type Output = String;
    type Error = StringError;

    async fn call(
        &self,
        _context: &mut ToolContext,
        args: Self::Args,
    ) -> Result<Self::Output, Self::Error> {
        let store = &self.vector_store;
        let category = args
            .category
            .unwrap_or_else(|| "explicit_user_fact".to_string());

        save_memory_snippet(
            &store,
            &category,
            &args.fact,
            self.conversation_id.as_deref(),
        )
        .await
        .map_err(|e| StringError(e.to_string()))?;

        Ok(format!(
            "Successfully saved to long-term memory: '{}'",
            args.fact
        ))
    }

    fn description(&self) -> String {
        "Stores a user's explicitly stated fact, preference, or detail into long-term memory for future recall.".to_string()
    }

    fn parameters(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "fact": {
                    "type": "string",
                    "description": "The exact fact, preference, or context snippet to remember."
                },
                "category": {
                    "type": "string",
                    "description": "Category tag, e.g., 'preference', 'project', 'personal', 'identity'."
                }
            },
            "required": ["fact"]
        })
    }
}
