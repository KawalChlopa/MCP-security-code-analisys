use rmcp::{handler::server::tool::{ToolRouter}, model::{CallToolResult, Content, Implementation, InitializeResult, ProtocolVersion, ServerCapabilities, ServerInfo}, tool, tool_handler, tool_router, ServerHandler};
use rmcp::ErrorData as McpError;


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
