use rig::tool::{Tool, ToolContext};
use serde::{Deserialize, Serialize};
use std::{fmt, process::Command};

#[derive(Debug)]
pub struct StringError(pub String);
impl fmt::Display for StringError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for StringError {}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct WebSearchArgs {
    pub query: String,
    pub max_results: u8,
}

#[derive(Deserialize, Debug)]
pub struct ScrapeResult {
    pub title: Option<String>,
    pub url: Option<String>,
    pub content: Option<String>,
    pub error: Option<String>,
}

pub struct WebSearchTool;

impl Tool for WebSearchTool {
    const NAME: &'static str = "web_search";

    type Error = StringError;
    type Args = WebSearchArgs;
    type Output = String;

    async fn call(
        &self,
        _context: &mut ToolContext, // USED for advanced too calls, not required here
        args: Self::Args,
    ) -> Result<Self::Output, Self::Error> {
        // Fallback to 3 if the LLM doesn't provide max_results or provides 0
        let max_results = if args.max_results == 0 {
            3
        } else {
            args.max_results
        };

        let jar_path = "java-scraper/target/scraper-1.0-SNAPSHOT-jar-with-dependencies.jar";

        let output = Command::new("java")
            .arg("-jar")
            .arg(jar_path)
            .arg(&args.query)
            .arg(max_results.to_string())
            .output()
            .map_err(|e| StringError(format!("Failed to execute Java process: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(StringError(format!("Java scraper failed: {}", stderr)));
        }

        let json_output = String::from_utf8_lossy(&output.stdout);

        let results: Vec<ScrapeResult> = serde_json::from_str(&json_output)
            .map_err(|e| StringError(format!("Failed to parse JSON: {}", e)))?;

        // Format the results into a clean Markdown string for the LLM to read
        let mut final_context = String::new();
        for (i, res) in results.iter().enumerate() {
            if let Some(err) = &res.error {
                final_context.push_str(&format!("### Source {}: Error\n{}\n\n", i + 1, err));
                continue;
            }

            let title = res.title.as_deref().unwrap_or("Untitled");
            let url = res.url.as_deref().unwrap_or("No URL");
            let content = res.content.as_deref().unwrap_or("No content extracted.");

            final_context.push_str(&format!(
                "### Source {}: {}\n**URL:** {}\n**Content:**\n{}\n\n---\n\n",
                i + 1,
                title,
                url,
                content
            ));
        }

        if final_context.is_empty() {
            Ok("No relevant search results found.".to_string())
        } else {
            Ok(final_context)
        }
    }

    fn description(&self) -> String {
        "Searches the web for real-time information. Use this when you need current facts, news, or specific research that goes beyond your training data. Returns structured Markdown summaries of the top search results.".to_string()
    }

    fn parameters(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "The search phrase to look up on the web."
                },
                "max_results": {
                    "type": "integer",
                    "description": "The number of top articles to read. Default to 3 for deep research."
                }
            },
            "required": ["query"]
        })
    }
}
