use rmcp::ErrorData as McpError;
use rmcp::{
    ServerHandler,
    handler::server::{tool::ToolRouter, wrapper::Parameters},
    model::{
        CallToolResult, Content, Implementation, InitializeResult, ProtocolVersion,
        ServerCapabilities, ServerInfo,
    },
    schemars, tool, tool_handler, tool_router,
};
use serde::{Deserialize, Serialize};
use std::path::Path;


// struct for scan rust files results
#[derive(Serialize)]
struct ScanRustResult {
    cargo_audit_output: CargoAuditOutput,
    cargo_clippy_output: CargoClippyOutput,
}


// import tools modules
use crate::tools::{bandit::BanditOutput, cargo_audit::CargoAuditOutput, cargo_clippy::CargoClippyOutput};

// Structs for parametrs
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ScanParams {
    path: String,
}

// Struct for tools and tool router
#[derive(Clone)]
pub struct SecurityMcpServer {
    // Problem with warning, tool router is used by macro, but dead code analisys take place before makros expansion
    #[allow(dead_code)]
    tool_router: ToolRouter<Self>,
}

// tools
#[tool_router]
impl SecurityMcpServer {
    // Tool router inicialization
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }

    // bandit python scanner
    #[tool(description = "Scan python code using file or directory with bandit")]
    async fn scan_python_bandit(
        &self,
        params: Parameters<ScanParams>,
    ) -> Result<CallToolResult, McpError> {
        let path = Path::new(&params.0.path);

        tracing::info!(path = %path.display(), "starting Bandit tool call");

        let result = BanditOutput::run_bandit(path)
            .await
            .map_err(|e| McpError::internal_error(format!("{e:#}"), None))?;

        // Switch result to json because Content::text dont know how to interpret object BanditOutput to text
        let result_json = serde_json::to_string_pretty(&result)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        Ok(CallToolResult::success(vec![Content::text(result_json)]))
    }

    // cargo rust scanner
    #[tool(description = "Scan rust code")]
    async fn scan_cargo_audit(
        &self,
        params: Parameters<ScanParams>,
    ) -> Result<CallToolResult, McpError> {
        let path = Path::new(&params.0.path);

        // Cargo Audit Scan
        let cargo_audit_output = CargoAuditOutput::run_cargo_audit(path)
            .await
            .map_err(|e| McpError::internal_error(format!("{e:#}"), None))?;

        // Cargo Clippy Scan
        let cargo_clippy_output = CargoClippyOutput::run_cargo_clippy(path)
            .await
            .map_err(|e| McpError::internal_error(format!("{e:#}"), None))?;

        // Create struct ScanRustResult
        let scan_rust_result = ScanRustResult{
            cargo_audit_output,
            cargo_clippy_output,
        };
        
        // Switch result to json because Content::text dont know how to interpret object CargoAuditOutput to text
        let result_json = serde_json::to_string_pretty(&scan_rust_result)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        Ok(CallToolResult::success(vec![Content::text(result_json)]))
    }
}

// tool handler which provides information for LLM
#[tool_handler]
impl ServerHandler for SecurityMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(
        ServerCapabilities::builder()
            .enable_tools()
            .build(),
    )
    .with_protocol_version(ProtocolVersion::LATEST)
    .with_server_info(Implementation::from_build_env())
    .with_instructions(
        "Security MCP server for scanning code. Available actions: scan Python with Bandit, scan Rust with cargo audit."
    )
    } // Initialization
    async fn initialize(
        &self,
        _request: rmcp::model::InitializeRequestParams,
        _context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> Result<InitializeResult, McpError> {
        Ok(self.get_info())
    }
}
