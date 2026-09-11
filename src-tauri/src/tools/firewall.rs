// firewall.rs (Rust integration - exposes firewall_manager.sh as an Atlas tool)
use rig::tool::{Tool, ToolContext};
use serde::{Deserialize, Serialize};
use std::{fmt, path::PathBuf, process::Command};

/// Resolves the installed Atlas tools directory.
/// Mirrors install.sh's default DEST ($HOME/.local/share/atlas/tools),
/// overridable via the ATLAS_TOOLS_DIR env var for custom install locations.
fn atlas_tools_dir() -> Result<PathBuf, StringError> {
    if let Ok(dir) = std::env::var("ATLAS_TOOLS_DIR") {
        return Ok(PathBuf::from(dir));
    }
    let home = std::env::var("HOME")
        .map_err(|_| StringError("HOME environment variable not set".into()))?;
    Ok(PathBuf::from(home).join(".local/share/atlas/tools"))
}

#[derive(Debug)]
pub struct StringError(pub String);
impl fmt::Display for StringError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl std::error::Error for StringError {}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct FirewallArgs {
    pub action: String, // status | allow | deny | ban | ...
    #[serde(default)]
    pub port: Option<String>, // "8080/tcp"
    #[serde(default)]
    pub ip: Option<String>, // "203.0.113.66"
    #[serde(default)]
    pub rate: Option<u8>, // new conns/min for ratelimit
}

pub struct FirewallTool;

impl Tool for FirewallTool {
    const NAME: &'static str = "firewall_manager";

    type Error = StringError;
    type Args = FirewallArgs;
    type Output = String;

    async fn call(
        &self,
        _context: &mut ToolContext, // used for advanced tool calls, not required here
        args: Self::Args,
    ) -> Result<Self::Output, Self::Error> {
        // 1. Resolve the installed script path and build the argv
        //    (passwordless sudo in Atlas)
        let tools_dir = atlas_tools_dir()?;
        let script = tools_dir.join("sudo").join("firewall_manager.sh");
        if !script.is_file() {
            return Err(StringError(format!(
                "firewall_manager.sh not found at {} (run install.sh first)",
                script.display()
            )));
        }

        let mut cmd = Command::new("sudo");
        cmd.arg("-n").arg("bash").arg(&script).arg(&args.action);
        if let Some(p) = &args.port {
            cmd.arg(p);
        }
        if let Some(ip) = &args.ip {
            cmd.arg(ip);
        }
        if let Some(r) = args.rate {
            cmd.arg(r.to_string());
        }

        // 2. Run it and capture stdout/stderr
        let output = cmd
            .output()
            .map_err(|e| StringError(format!("Failed to execute firewall script: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(StringError(format!("firewall_manager failed: {}", stderr)));
        }

        // 3. Contract: the LAST stdout line is a JSON status object
        let stdout = String::from_utf8_lossy(&output.stdout);
        let json = stdout
            .lines()
            .rev()
            .find(|l| l.trim_start().starts_with('{'))
            .ok_or_else(|| StringError("No JSON payload from firewall_manager".into()))?;

        Ok(json.to_string())
    }

    fn description(&self) -> String {
        "Manage the host firewall: check status, allow or deny ports, ban or \
         unban malicious IPs, rate-limit brute-forced services, back up rules \
         and produce a security posture report. Only use for defensive \
         operations explicitly requested by the user."
            .to_string()
    }

    fn parameters(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "description": "One of: status, list, allow, deny, ban, unban, \
                                    banlist, ratelimit, unratelimit, backup, \
                                    restore, report, watch, logs."
                },
                "port": {
                    "type": "string",
                    "description": "PORT[/tcp|udp], e.g. 8080/tcp (allow/deny/ratelimit)."
                },
                "ip": {
                    "type": "string",
                    "description": "IPv4 or IPv6 address (ban/unban)."
                },
                "rate": {
                    "type": "integer",
                    "description": "New connections per minute for ratelimit. Default 6."
                }
            },
            "required": ["action"]
        })
    }
}
