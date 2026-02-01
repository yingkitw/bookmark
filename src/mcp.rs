use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};
use std::path::PathBuf;

use crate::browser::Browser;
use crate::exporter::{export_data, Bookmark, UrlEntry};
use crate::graph::{GraphBuilder, GraphConfig};
use crate::processor::{BookmarkProcessor, ProcessingConfig};
use crate::search::{search_bookmarks_internal, SearchOptions};

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

    fn tool_export_bookmarks(&self, args: Value) -> Result<Value> {
        let browser = args["browser"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing browser"))?;
        let data_type = args["data_type"].as_str().unwrap_or("bookmarks");

        let temp_dir = PathBuf::from("/tmp/bookmark_mcp");
        std::fs::create_dir_all(&temp_dir)?;

        let output_file = temp_dir.join(format!("{}-{}.yaml", browser, data_type));
        export_data(browser, data_type, Some(output_file.clone()), None)?;

        let content = std::fs::read_to_string(&output_file)?;
        let data: Vec<crate::exporter::BrowserData> = serde_yaml::from_str(&content)?;

        Ok(json!({
            "content": [{
                "type": "text",
                "text": serde_json::to_string_pretty(&data)?
            }]
        }))
    }

    fn tool_search_bookmarks(&self, args: Value) -> Result<Value> {
        let query = args["query"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing query"))?;
        let title_only = args["title_only"].as_bool().unwrap_or(false);
        let url_only = args["url_only"].as_bool().unwrap_or(false);
        let limit = args["limit"].as_u64().unwrap_or(20) as usize;

        let options = SearchOptions {
            title_only,
            url_only,
            limit,
        };

        let results = search_bookmarks_internal(query, &options)?;

        let formatted_results: Vec<String> = results
            .iter()
            .map(|b| format!("{} - {}", b.title, b.url.as_deref().unwrap_or("N/A")))
            .collect();

        Ok(json!({
            "content": [{
                "type": "text",
                "text": formatted_results.join("\n")
            }]
        }))
    }

    fn tool_list_browsers(&self, args: Value) -> Result<Value> {
        let mut output = Vec::new();

        let browsers = if let Some(browser_name) = args["browser"].as_str() {
            vec![browser_name]
        } else {
            vec!["chrome", "firefox", "safari", "edge"]
        };

        for browser_name in browsers {
            if let Ok(browser) = Browser::from_str(browser_name) {
                if let Ok(profiles) = browser.find_profiles(None) {
                    output.push(format!(
                        "{}: {} profile(s)",
                        browser_name,
                        profiles.len()
                    ));
                }
            }
        }

        Ok(json!({
            "content": [{
                "type": "text",
                "text": output.join("\n")
            }]
        }))
    }

    fn tool_process_bookmarks(&self, args: Value) -> Result<Value> {
        let bookmarks_json = args["bookmarks"]
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("Missing bookmarks array"))?;
        let mode = args["mode"].as_str().unwrap_or("both");
        let strategy = args["strategy"].as_str().unwrap_or("merge");

        let bookmarks: Vec<Bookmark> = serde_json::from_value(json!(bookmarks_json))?;

        let merge_strategy = match strategy {
            "first" => crate::deduplication::MergeStrategy::KeepFirst,
            "last" => crate::deduplication::MergeStrategy::KeepLast,
            "recent" => crate::deduplication::MergeStrategy::KeepMostRecent,
            _ => crate::deduplication::MergeStrategy::MergeMetadata,
        };

        let config = ProcessingConfig {
            deduplication_config: crate::deduplication::DeduplicationConfig {
                merge_strategy,
                normalize_urls: mode == "dedupe" || mode == "both",
                ..Default::default()
            },
            organization_config: crate::organization::OrganizationConfig::default(),
            dry_run: false,
            backup_original: false,
        };

        let processor = BookmarkProcessor::new(config);
        let result = processor.process_bookmarks(&bookmarks)?;

        Ok(json!({
            "content": [{
                "type": "text",
                "text": format!(
                    "Processed {} bookmarks:\n- Original: {}\n- Final: {}\n- Duplicates removed: {}",
                    result.processing_summary.original_count,
                    result.processing_summary.original_count,
                    result.processing_summary.final_count,
                    result.processing_summary.duplicates_removed
                )
            }],
            "processed_bookmarks": result.processed_bookmarks
        }))
    }

    fn tool_generate_graph(&self, args: Value) -> Result<Value> {
        let browser = args["browser"].as_str().unwrap_or("all");
        let format = args["format"].as_str().unwrap_or("json");
        let min_threshold = args["min_threshold"].as_u64().unwrap_or(2) as usize;

        let temp_dir = PathBuf::from("/tmp/bookmark_mcp");
        std::fs::create_dir_all(&temp_dir)?;

        let data_type = "both";
        let output_file = temp_dir.join(format!("{}-{}.yaml", browser, data_type));
        export_data(browser, data_type, Some(output_file.clone()), None)?;

        let content = std::fs::read_to_string(&output_file)?;
        let browser_data: Vec<crate::exporter::BrowserData> = serde_yaml::from_str(&content)?;

        let mut all_bookmarks = Vec::new();
        let mut all_history = Vec::new();

        for data in browser_data {
            if let Some(b) = data.bookmarks {
                all_bookmarks.extend(b);
            }
            if let Some(h) = data.history {
                all_history.extend(h.urls);
            }
        }

        let config = GraphConfig {
            min_domain_threshold: min_threshold,
            ..Default::default()
        };

        let mut builder = GraphBuilder::new(config);
        let graph = builder.from_both(&all_bookmarks, &all_history)?;

        let graph_content = match format {
            "dot" => crate::graph::formats::to_dot(&graph),
            "json" => crate::graph::formats::to_json(&graph),
            "gexf" => crate::graph::formats::to_gexf(&graph),
            _ => return Err(anyhow::anyhow!("Invalid format")),
        };

        Ok(json!({
            "content": [{
                "type": "text",
                "text": graph_content
            }],
            "metadata": {
                "nodes": graph.metadata.total_nodes,
                "edges": graph.metadata.total_edges,
                "bookmarks": graph.metadata.bookmark_count,
                "domains": graph.metadata.domain_count
            }
        }))
    }
}

impl Default for McpServer {
    fn default() -> Self {
        Self::new()
    }
}
