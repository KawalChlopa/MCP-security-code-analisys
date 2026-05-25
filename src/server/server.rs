use rmcp::{ServerHandler, handler::server::{tool::ToolRouter, wrapper::Parameters}, model::{CallToolResult, Content, Implementation, InitializeResult, ProtocolVersion, ServerCapabilities, ServerInfo}, tool, tool_handler, tool_router, schemars};
use rmcp::ErrorData as McpError;
use serde::Deserialize;

// import tools modules
use crate::tools::{bandit::BanditOutput, cargo_audit::CargoAuditOutput};


// Structs for parametrs
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ScanParams {
    path: String,
}

// Struct for tools and tool router
#[derive(Clone)]
pub struct SecurityMcpServer {
    tool_router: ToolRouter<Self>
}

// tools
#[tool_router]
impl SecurityMcpServer {
    // Tool router inicialization
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router()
        }
    }


    // bandit python scanner
    #[tool(description = "Scan python code using file or directory with bandit")]
    async fn scan_python_bandit(&self, params: Parameters<ScanParams>) -> Result<CallToolResult, McpError> {
        let path = params.0.path;

        eprintln!("Starting bandit scan...");

        let result = BanditOutput::run_bandit(&path)
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        // Switch result to json because Content::text dont know how to interpret object BanditOutput to text
        let result_json = serde_json::to_string_pretty(&result)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        Ok(CallToolResult::success(vec![Content::text(result_json)]))
    }
    
    // cargo rust scanner
    #[tool(description = "Scan rust code")]
    async fn scan_cargo_audit(&self, params: Parameters<ScanParams>) -> Result<CallToolResult, McpError> {
        let path = params.0.path;
        let result = CargoAuditOutput::run_cargo_audit(&path)
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        // Switch result to json because Content::text dont know how to interpret object CargoAuditOutput to text
        let result_json = serde_json::to_string_pretty(&result)
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
}    // Initialization
    async fn initialize(
        &self,
        _request: rmcp::model::InitializeRequestParam,
        _context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> Result<InitializeResult, McpError> {
        Ok(self.get_info())
    }
}
