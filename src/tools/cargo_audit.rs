use std::process::Stdio;

use anyhow::Context;
use serde::{Deserialize, Serialize};
use tokio::process::Command;

// Struct for output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CargoAuditOutput {
    vulnerabilities: Vulnerabilities,
    warnings: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Vulnerabilities {
    found: bool,
    count: i32,
    list: Vec<String>,
}

impl CargoAuditOutput {
    // cargo audit run
    pub async fn run_cargo_audit(path: &str) -> Result<Self, anyhow::Error> {
        let output = Command::new("cargo")
            .arg("audit")
            .current_dir(&path)
            .stdin(Stdio::null()) // This line is critical otherwise process will inheret stdin and claude will not be able to communicate with MCP
            .output()
            .await
            .context("Failed to run cargo audit")?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        // check if result is stdout is empty
        if stdout.trim().is_empty() {
            anyhow::bail!("Bandit reuturned empty result. Error: {}", stderr);
        }

        // try to parse result
        let result: CargoAuditOutput =
            serde_json::from_str(&stdout).context("Failed to parse Bandit JSON output")?;

        Ok(result)
    }
}
