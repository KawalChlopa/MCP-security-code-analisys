use std::{env, path::Path, process::Stdio, time::Duration};
use anyhow::Context;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::process::Command;



#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CargoClippyFixOutput {
    success: bool,
    stdout: String,
    stderr: String,
    exit_code: Option<i32>,
    changed: bool,
    diff: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CargoClippyOutput {
    messages: Vec<Value>,
    stderr: String,
    success: bool,
    exit_code: Option<i32>,
}




impl CargoClippyOutput {
    pub async fn run_cargo_clippy (path: &Path) -> Result<Self, anyhow::Error> {
        // Clippy scan command
        let output = Command::new("cargo")
            .args(["clippy", "--all-targets", "--message-format=json"]) // we check all files and output json
            .current_dir(path)
            .stdin(Stdio::null()) // This line is critical otherwise process will inheret stdin and claude will not be able to communicate with MCP
            .kill_on_drop(true) // if we lose the handle process will be killed
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
            exit_code: output.status.code(),
        }
    )
    }
}

impl CargoClippyFixOutput {

    pub async fn run_cargo_clippy_fix (path: &Path) -> Result<Self, anyhow::Error> {
        // Clippy fix command
        // allow dirty - allows to change your files even if there are uncommited changes in files
        // allow staged - allows to change files even if there are staged
        let output = Command::new("cargo")
            .args(["clippy", "--fix", "--allow-dirty", "--allow-staged"])
            .current_dir(path)
            .stdin(Stdio::null()) // This line is critical otherwise process will inheret stdin and claude will not be able to communicate with MCP
            .kill_on_drop(true) // if we lose the handle process will be killed
            .output()
            .await
            .with_context(|| format!("Failed to run cargo clippy in {}", path.display()))?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let clippy_success = output.status.success();

        // Run git diff --quiet to check if clippy do some changes or not
        let diff_check = Command::new("git")
            .args(["diff","--quiet"])
            .current_dir(path)
            .stdin(Stdio::null())
            .kill_on_drop(true)
            .output()
            .await 
            .with_context(|| format!("Failed to run git diff in {}", path.display()))?;

        let has_changes = match diff_check.status.code() {
            Some(0) => false,
            Some(1) => true,
            _ => anyhow::bail!("git diff --quiet ended with error: {}", String::from_utf8_lossy(&diff_check.stderr)),
        };

        // if has changes true retunr changes if not return empty string
        let diff = if has_changes {
            let diff_output = Command::new("git")
                .arg("diff")
                .current_dir(path)
                .stdin(Stdio::null())
                .kill_on_drop(true)
                .output()
                .await
                .with_context(|| format!("Failed to run git diff in {}", path.display()))?;

            String::from_utf8_lossy(&diff_output.stdout).to_string()       
        } else {
            String::new()
        };

        Ok(Self {
            success: clippy_success,
            stdout,
            stderr,
            exit_code: output.status.code(),
            changed: has_changes,
            diff,
        })

    }
}