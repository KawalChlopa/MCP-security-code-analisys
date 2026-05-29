use std::{env, path::Path, process::Stdio, time::Duration};
use anyhow::Context;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::process::Command;



#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CargoClippyOutput{
    messages: Vec<Value>,
    stderr: String,
    success: bool,
}




impl CargoClippyOutput {
    pub async fn run_cargo_clippy (path: &Path) -> Result<Self, anyhow::Error> {
        let output = Command::new("cargo")
            .args(["clippy", "--all-targets", "--message-format=json"]) // we check all files and output json
            .current_dir(path)
            .stdin(Stdio::null()) // This line is critical otherwise process will inheret stdin and claude will not be able to communicate with MCP
            .kill_on_drop(true)
            .output()
            .await
            .with_context(|| format!("Failed to run cargo audit in {}", path.display()))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        // check if result stdout is empoty
        if stdout.trim().is_empty() {
            anyhow::bail!("cargo clippy returned empty result. stderr: {}", stderr);
        }

        // try to parse jsons because there is a lot of them in output 
        let mut messages = Vec::new();

        for line in stdout.lines() {
            let line = line.trim();
            
            // skip empty lines
            if line.is_empty() {
                continue;
            }

            let value: Value = serde_json::from_str(line)
                .with_context(|| format!("Failed to parse cargo clippy JSON line: {}", line))?;
            messages.push(value);
        }
        Ok(Self {
            messages,
            stderr: stderr.to_string(),
            success: output.status.success(),
        }
    )
    }
}