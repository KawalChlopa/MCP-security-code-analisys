use std::{env, path::Path, process::Stdio, time::Duration};

use anyhow::{Context, anyhow};
use serde::{Deserialize, Serialize};
use tokio::{process::Command, time::timeout};

const DEFAULT_BANDIT_TIMEOUT_SECONDS: u64 = 60;

// Struct for bandit results, which will cantain vector with issues
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BanditOutput {
    errors: Vec<String>,
    #[serde(rename = "generated_at")]
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
        let python = env::var("BANDIT_PYTHON").unwrap_or_else(|_| "python".to_owned());
        let timeout_duration = bandit_timeout();
        let mut cmd = Command::new(&python);
        cmd.args(["-m", "bandit"]);
        cmd.stdin(Stdio::null());
        cmd.kill_on_drop(true);

        // Change to Path
        let scan_path = Path::new(path);

        // We want to check if file or dir exists
        if !scan_path.exists() {
            anyhow::bail!("Path does not exist: {}", path);
        }

        if scan_path.is_dir() {
            cmd.arg("-r");
        }

        tracing::debug!(path, python, "starting Bandit scan");

        let output = timeout(
            timeout_duration,
            cmd.arg(scan_path).args(["-f", "json"]).output(),
        )
        .await
        .map_err(|_| {
            anyhow!(
                "Bandit scan timed out after {} seconds for path: {}",
                timeout_duration.as_secs(),
                path
            )
        })?
        .with_context(|| {
            format!(
                "Failed to run Bandit with `{python} -m bandit`; set BANDIT_PYTHON to the Python executable that has Bandit installed"
            )
        })?;

        tracing::debug!(path, "Bandit scan finished");
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        // exit code 2 = bandit internal error
        if output.status.code() == Some(2) {
            anyhow::bail!("Bandit error (exit 2): {}", stderr);
        }

        // check if result is stdout is empty
        if stdout.trim().is_empty() {
            anyhow::bail!("Bandit returned an empty result. Error: {}", stderr);
        }

        // try to parse result
        let result: BanditOutput =
            serde_json::from_str(&stdout).context("Failed to parse Bandit JSON output")?;

        Ok(result) // return result
    }
}

fn bandit_timeout() -> Duration {
    let seconds = env::var("BANDIT_TIMEOUT_SECONDS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .filter(|seconds| *seconds > 0)
        .unwrap_or(DEFAULT_BANDIT_TIMEOUT_SECONDS);

    Duration::from_secs(seconds)
}
