use anyhow::Result;
use bookmark::mcp::McpServer;

fn main() -> Result<()> {
    env_logger::init();
    
    let server = McpServer::new();
    server.run()
}
