use std::{env, path::Path, process::Stdio, time::Duration};

use anyhow::Context;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::process::Command;

// Struct for output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CargoAuditOutput {
    vulnerabilities: Vulnerabilities,
    warnings: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Vulnerabilities {
    found: bool,
    count: i32,
    list: Vec<Value>,
}

impl CargoAuditOutput {
    // cargo audit run
    pub async fn run_cargo_audit(path: &str) -> Result<Self, anyhow::Error> {
        // change to path
        let scan_path = Path::new(path);

        // output running command
        let output = Command::new("cargo")
            .args(["audit", "--json"])
            .current_dir(&scan_path)
            .stdin(Stdio::null()) // This line is critical otherwise process will inheret stdin and claude will not be able to communicate with MCP
            .kill_on_drop(true) 
            .output()
            .await
            .context("Failed to run cargo audit")?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        // check if result is stdout is empty
        if stdout.trim().is_empty() {
            anyhow::bail!("cargo audit returned empty result. stderr: {}", stderr);
        }

        // try to parse result
        let result: CargoAuditOutput =
            serde_json::from_str(&stdout).context("Failed to parse cargo audit JSON output")?;

        Ok(result)
    }
}
