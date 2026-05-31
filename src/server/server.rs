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



use crate::server::helpers;

// Structs for parametrs
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ScanParams {
    path: String,
    action: String,
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
        Parameters(params): Parameters<ScanParams>,
    ) -> Result<CallToolResult, McpError> {
        let path = Path::new(&params.path);

        tracing::info!(path = %path.display(), "starting Bandit tool call");

        // result from python handler
        let result_json = helpers::handle_python_bandit(path)
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        Ok(CallToolResult::success(vec![Content::text(result_json)]))
    }

    // cargo rust scanner
    #[tool(description = "Scan or fix rust project")]
    async fn scan_cargo(
        &self,
        Parameters(params): Parameters<ScanParams>,
    ) -> Result<CallToolResult, McpError> {

        let path = Path::new(&params.path);

        // match proper action
        let result_json = match params.action.as_str() {

            "fix"  => helpers::handle_rust_fix(path)
                        .await
                        .map_err(|e| McpError::internal_error(e.to_string(), None))?,

            "scan" => helpers::handle_rust_scan(path)
                        .await
                        .map_err(|e| McpError::internal_error(e.to_string(), None))?,

            other  => return Err(McpError::invalid_params(format!("Unknown action {other}, Use action: fix or scan"), None)),
        };

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
        "Security MCP server for scanning code. Available actions: scan Python with Bandit, scan Rust with cargo audit and clippy, fix Rust code with cargo clippy fix."
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
