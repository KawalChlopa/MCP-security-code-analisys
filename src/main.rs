// modules
mod server;
mod tools;


use anyhow::Result;
use rmcp::{ServiceExt, transport::stdio};
use tracing_subscriber::EnvFilter;

// Importing the MCP Server
use crate::server::server::SecurityMcpServer;

#[tokio::main]
async fn main() -> Result <()> {
    // Initialiazing logs
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::from_default_env()
                .add_directive(tracing::Level::DEBUG.into())
        )
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    // Show log message
    tracing::info!("Starting MCP Server");


    // Create an instance of our MCP Server and start serving on STDIO
    let service = SecurityMcpServer::new().serve(stdio()).await.inspect_err(|e| {
        tracing::error!("serving error: {:?}", e)
    })?;

    // Run ther service
    service.waiting().await?; // wait till server is runing

    Ok(())
}