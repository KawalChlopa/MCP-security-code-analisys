use rmcp::{ServerHandler, handler::server::{tool::ToolRouter, wrapper::Parameters}, model::{CallToolResult, Content, Implementation, InitializeResult, ProtocolVersion, ServerCapabilities, ServerInfo}, tool, tool_handler, tool_router, schemars};
use rmcp::ErrorData as McpError;
use serde::Deserialize;

// import tools modules
use crate::tools::bandit::BanditOutput;


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
    #[tool(description = "Scan python code using Bandit")]
    async fn scan_python_bandit(&self, params: Parameters<ScanParams>) -> Result<CallToolResult, McpError> {
        let path = params.0.path;
        let result = BanditOutput::run_bandit(&path)
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        // Switch result to json because Content::text dont know how to interpret object BanditOutput to text
        let result_json = serde_json::to_string_pretty(&result)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        Ok(CallToolResult::success(vec![Content::text(result_json)]))
    }
    
    // cargo rust scanner
}


// tool handler which provides information for LLM
#[tool_handler]
impl ServerHandler for SecurityMcpServer {
    fn get_info(&self) -> rmcp::model::ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2025_06_18,
            capabilities: ServerCapabilities::builder()
                .enable_tools() // Shows enable tools
                .build(),
            server_info: Implementation::from_build_env(), // Infor on the MCP from the build_env
            instructions: Some( // Instructions sent back to the MCP client - system prompt
            "I will scan your code in order to find any vulnerabilities.
            
            
            Available actions: ".to_string()),
        }
    }
    // Initialization
    async fn initialize(
        &self,
        _request: rmcp::model::InitializeRequestParam,
        _context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> Result<InitializeResult, McpError> {
        Ok(self.get_info())
    }
}
