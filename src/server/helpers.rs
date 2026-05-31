use rmcp::ErrorData as McpError;
use crate::tools::{bandit::BanditOutput, cargo_audit::CargoAuditOutput, cargo_clippy::{CargoClippyFixOutput, CargoClippyOutput}};
use std::path::Path;
use serde::Serialize;


// struct for scan rust results
#[derive(Serialize)]
struct ScanRustResult {
    cargo_audit_output: CargoAuditOutput,
    cargo_clippy_output: CargoClippyOutput,
}


// Funtion for handle rust cargo clippy scan output
pub async fn handle_rust_scan(path: &Path) -> Result<String, anyhow::Error> {
    let cargo_audit_output = CargoAuditOutput::run_cargo_audit(path)
                    .await
                    .map_err(|e| McpError::internal_error(format!("{e:#}"), None))?;

                let cargo_clippy_output = CargoClippyOutput::run_cargo_clippy(path)
                    .await
                    .map_err(|e| McpError::internal_error(format!("{e:#}"), None))?;

                let scan_rust_result = ScanRustResult{
                    cargo_audit_output,
                    cargo_clippy_output,
                };
        
                // Switch result to json because Content::text dont know how to interpret object CargoAuditOutput to text
                let result = serde_json::to_string_pretty(&scan_rust_result)
                    .map_err(|e| McpError::internal_error(e.to_string(), None))?;

                Ok(result)
}


// Funtion for handle rust cargo clippy fix output
pub async fn handle_rust_fix(path: &Path) -> Result<String, anyhow::Error> {
    let cargo_clippy_fix_output = CargoClippyFixOutput::run_cargo_clippy_fix(path)
                    .await
                    .map_err(|e| McpError::internal_error(e.to_string(), None))?;
                
                let result = serde_json::to_string_pretty(&cargo_clippy_fix_output)
                    .map_err(|e| McpError::internal_error(e.to_string(), None))?;

                Ok(result)
}

// Function for handle python bandit scan output
pub async fn handle_python_bandit(path: &Path) -> Result<String, anyhow::Error> {
    let bandit_output = BanditOutput::run_bandit(path)
        .await
        .map_err(|e| McpError::internal_error(format!("{e:#}"), None))?;

        // Switch result to json because Content::text dont know how to interpret object BanditOutput to text
    let result = serde_json::to_string_pretty(&bandit_output)
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;

    Ok(result)
    
}