mod tools;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};

#[derive(Debug, Serialize, Deserialize)]
struct McpRequest {
    jsonrpc: String,
    id: Option<Value>,
    method: String,
    params: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
struct McpResponse {
    jsonrpc: String,
    id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<McpError>,
}

#[derive(Debug, Serialize, Deserialize)]
struct McpError {
    code: i32,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
}

pub struct McpServer {
    name: String,
    version: String,
}

impl McpServer {
    pub fn new() -> Self {
        Self {
            name: "bookmark-mcp".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }

    pub fn run(&self) -> Result<()> {
        log::info!("Starting MCP server: {} v{}", self.name, self.version);

        let stdin = io::stdin();
        let mut stdout = io::stdout();

        for line in stdin.lock().lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }

            let response = match serde_json::from_str::<McpRequest>(&line) {
                Ok(request) => self.handle_request(request),
                Err(e) => McpResponse {
                    jsonrpc: "2.0".to_string(),
                    id: None,
                    result: None,
                    error: Some(McpError {
                        code: -32700,
                        message: format!("Parse error: {}", e),
                        data: None,
                    }),
                },
            };

            let response_json = serde_json::to_string(&response)?;
            writeln!(stdout, "{}", response_json)?;
            stdout.flush()?;
        }

        Ok(())
    }

    fn handle_request(&self, request: McpRequest) -> McpResponse {
        log::debug!("Handling request: {:?}", request.method);

        let result = match request.method.as_str() {
            "initialize" => self.handle_initialize(request.params),
            "tools/list" => self.handle_list_tools(),
            "tools/call" => self.handle_tool_call(request.params),
            _ => Err(anyhow::anyhow!("Method not found: {}", request.method)),
        };

        match result {
            Ok(value) => McpResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: Some(value),
                error: None,
            },
            Err(e) => McpResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: None,
                error: Some(McpError {
                    code: -32603,
                    message: e.to_string(),
                    data: None,
                }),
            },
        }
    }

    fn handle_initialize(&self, _params: Option<Value>) -> Result<Value> {
        Ok(json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "tools": {}
            },
            "serverInfo": {
                "name": self.name,
                "version": self.version
            }
        }))
    }

    fn handle_list_tools(&self) -> Result<Value> {
        Ok(json!({
            "tools": [
                {
                    "name": "export_bookmarks",
                    "description": "Export bookmarks from a browser",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "browser": {
                                "type": "string",
                                "description": "Browser name (chrome, firefox, safari, edge, all)",
                                "enum": ["chrome", "firefox", "safari", "edge", "all"]
                            },
                            "data_type": {
                                "type": "string",
                                "description": "Data type to export",
                                "enum": ["bookmarks", "history", "both"],
                                "default": "bookmarks"
                            }
                        },
                        "required": ["browser"]
                    }
                },
                {
                    "name": "search_bookmarks",
                    "description": "Search bookmarks by query",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "query": {
                                "type": "string",
                                "description": "Search query"
                            },
                            "title_only": {
                                "type": "boolean",
                                "description": "Search in title only",
                                "default": false
                            },
                            "url_only": {
                                "type": "boolean",
                                "description": "Search in URL only",
                                "default": false
                            },
                            "limit": {
                                "type": "integer",
                                "description": "Maximum number of results",
                                "default": 20
                            }
                        },
                        "required": ["query"]
                    }
                },
                {
                    "name": "list_browsers",
                    "description": "List available browsers and their profiles",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "browser": {
                                "type": "string",
                                "description": "Specific browser to list (optional)",
                                "enum": ["chrome", "firefox", "safari", "edge"]
                            }
                        }
                    }
                },
                {
                    "name": "process_bookmarks",
                    "description": "Deduplicate and organize bookmarks",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "bookmarks": {
                                "type": "array",
                                "description": "Array of bookmarks to process"
                            },
                            "mode": {
                                "type": "string",
                                "description": "Processing mode",
                                "enum": ["dedupe", "organize", "both"],
                                "default": "both"
                            },
                            "strategy": {
                                "type": "string",
                                "description": "Merge strategy for duplicates",
                                "enum": ["first", "last", "recent", "merge"],
                                "default": "merge"
                            }
                        },
                        "required": ["bookmarks"]
                    }
                },
                {
                    "name": "generate_graph",
                    "description": "Generate knowledge graph from bookmarks",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "browser": {
                                "type": "string",
                                "description": "Browser source",
                                "enum": ["chrome", "firefox", "safari", "edge", "all"],
                                "default": "all"
                            },
                            "format": {
                                "type": "string",
                                "description": "Output format",
                                "enum": ["dot", "json", "gexf"],
                                "default": "json"
                            },
                            "min_threshold": {
                                "type": "integer",
                                "description": "Minimum bookmarks for domain node",
                                "default": 2
                            }
                        }
                    }
                }
            ]
        }))
    }

    fn handle_tool_call(&self, params: Option<Value>) -> Result<Value> {
        let params = params.ok_or_else(|| anyhow::anyhow!("Missing params"))?;
        let tool_name = params["name"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing tool name"))?;
        let arguments = params["arguments"].clone();

        match tool_name {
            "export_bookmarks" => self.tool_export_bookmarks(arguments),
            "search_bookmarks" => self.tool_search_bookmarks(arguments),
            "list_browsers" => self.tool_list_browsers(arguments),
            "process_bookmarks" => self.tool_process_bookmarks(arguments),
            "generate_graph" => self.tool_generate_graph(arguments),
            _ => Err(anyhow::anyhow!("Unknown tool: {}", tool_name)),
        }
    }
}

impl Default for McpServer {
    fn default() -> Self {
        Self::new()
    }
}
