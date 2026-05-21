use std::process::Command;
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
        let output = Command::new("bandit")
            .args(["-r", path, "-f", "json"])
            .output()
            .context("Failed to run bandit")?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        // check if result is stdout is empty
        if stdout.trim().is_empty() {
            anyhow::bail!("Bandit reuturned empty result. Error: {}", stderr);
        }
        
        // try to parse result
        let result: BanditOutput = serde_json::from_str(&stdout)
            .context("Failed to parse Bandit JSON outup")?;

        Ok(result)  // return result 
    }

}