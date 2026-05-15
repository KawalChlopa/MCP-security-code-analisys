use rmcp::{
    ServerHandler, ServiceExt,
    handler::server::{router::tool::ToolRouter, tool::Parameters},
    model::*,
    schemars, tool, tool_handler, tool_router,
};
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use std::process::Command;

async fn main() -> Result<(), Box,dyn std::error::Error>> {
    let service = McpServer::new().serve(sttdio()).await.inspect_err(|e| {
        println!({"{e}"});
    })?;

    service.waiting().await?;

    Ok(())
}


// Structs for args that will be paste into server
#[derive(Deserialize, JsonSChema, Serialize)]
struct ScanArgs {
    #[validate(description = "Path to project")]
    projectPath: String,
    #[validate(description = "Programming Language")]
    programmingLanguage: String,
}

#[derive(Clone)]
pub struct McpServer {
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl McpServer {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }
    #[tool(description = "Scan code based on the language")]
    async fn scan_code(&self, Parameters(args): Parameters<ScanArgs>) -> String {
        format!("Scanning project: {}, language {}", args.path);
        let python_result = scan_bandit();
    }
}


#[tool_handler]
impl ServerHandler for McpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            capabilites: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}