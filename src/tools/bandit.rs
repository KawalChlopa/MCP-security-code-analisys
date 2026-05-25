use std::path::Path;
use tokio::process::Command;
use anyhow::Context;
use serde::{Deserialize, Serialize};



// Struct for bandit results, which will cantain vector with issues
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BanditOutput {
    errors: Vec<String>,
    #[serde(rename="generated_at")]
    time: String,
    results: Vec<Results>,

}

// Struct for bandit issues
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Results {
    code: String,
    filename: String,
    issue_confidence: String,
    issue_severity: String,
    issue_cwe: IssueCwe,
    issue_text: String,
    line_number: i32,
    line_range: Vec<i32>,
    more_info: String,
    test_name: String,
    test_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct IssueCwe {
    id: i32,
    link: String,
}

impl BanditOutput {
    // Run bandit function
    pub async fn run_bandit(path: &str) -> Result<Self, anyhow::Error> {
        let mut cmd = Command::new("python");
        cmd.args(["-m", "bandit"]);

        // Change to Path
        let scan_path = Path::new(path);
        
        // We want to check if file or dir exists
        if !scan_path.exists() {
            anyhow::bail!("Path does not exist: {}", path);
        }

        if scan_path.is_dir() {
            cmd.arg("-r");
        }

        // Logs added for debuging
        eprintln!("DEBUG: Starting scan for path: {}", path);

        // Claude has problem with calling bandit directly even if path is set so I want to test py
        let output = cmd
            .arg(scan_path)
            .args(["-f", "json"])
            .output()
            .await
            .context("Failed to run bandit")?;
        
        eprintln!("DEBUG: Bandit finished");
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

         // exit code 2 = bandit internal error
        if output.status.code() == Some(2) {
            anyhow::bail!("Bandit error (exit 2): {}", stderr);
        }

        // check if result is stdout is empty
        if stdout.trim().is_empty() {
            anyhow::bail!("Bandit reuturned empty result. Error: {}", stderr);
        }
        
        // try to parse result
        let result: BanditOutput = serde_json::from_str(&stdout)
            .context("Failed to parse Bandit JSON output")?;
        
        eprintln!("DEBUG: Returning result");

        Ok(result)  // return result 
    }

}
