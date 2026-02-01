# MCP Server Documentation

## Overview

The Bookmark MCP (Model Context Protocol) server provides AI assistants with tools to manage browser bookmarks through a standardized JSON-RPC 2.0 interface.

## Building

```bash
cargo build --release --features mcp --bin bookmark-mcp
```

## Running

```bash
./target/release/bookmark-mcp
```

The server communicates via stdin/stdout using JSON-RPC 2.0 protocol.

## Protocol

### Initialize

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "initialize",
  "params": {}
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "protocolVersion": "2024-11-05",
    "capabilities": {
      "tools": {}
    },
    "serverInfo": {
      "name": "bookmark-mcp",
      "version": "0.1.2"
    }
  }
}
```

### List Tools

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "tools/list",
  "params": {}
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "result": {
    "tools": [
      {
        "name": "export_bookmarks",
        "description": "Export bookmarks from a browser",
        "inputSchema": { ... }
      },
      ...
    ]
  }
}
```

## Available Tools

### 1. export_bookmarks

Export bookmarks from a specific browser.

**Parameters:**
- `browser` (required): Browser name (chrome, firefox, safari, edge, all)
- `data_type` (optional): Data type to export (bookmarks, history, both)

**Example:**
```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "tools/call",
  "params": {
    "name": "export_bookmarks",
    "arguments": {
      "browser": "chrome",
      "data_type": "bookmarks"
    }
  }
}
```

### 2. search_bookmarks

Search bookmarks by query string.

**Parameters:**
- `query` (required): Search query
- `title_only` (optional): Search in title only (default: false)
- `url_only` (optional): Search in URL only (default: false)
- `limit` (optional): Maximum results (default: 20)

**Example:**
```json
{
  "jsonrpc": "2.0",
  "id": 4,
  "method": "tools/call",
  "params": {
    "name": "search_bookmarks",
    "arguments": {
      "query": "github",
      "limit": 10
    }
  }
}
```

### 3. list_browsers

List available browsers and their profiles.

**Parameters:**
- `browser` (optional): Specific browser to list

**Example:**
```json
{
  "jsonrpc": "2.0",
  "id": 5,
  "method": "tools/call",
  "params": {
    "name": "list_browsers",
    "arguments": {}
  }
}
```

### 4. process_bookmarks

Deduplicate and organize bookmarks.

**Parameters:**
- `bookmarks` (required): Array of bookmark objects
- `mode` (optional): Processing mode (dedupe, organize, both)
- `strategy` (optional): Merge strategy (first, last, recent, merge)

**Example:**
```json
{
  "jsonrpc": "2.0",
  "id": 6,
  "method": "tools/call",
  "params": {
    "name": "process_bookmarks",
    "arguments": {
      "bookmarks": [...],
      "mode": "both",
      "strategy": "merge"
    }
  }
}
```

### 5. generate_graph

Generate knowledge graph from bookmarks.

**Parameters:**
- `browser` (optional): Browser source (default: all)
- `format` (optional): Output format (dot, json, gexf) (default: json)
- `min_threshold` (optional): Minimum bookmarks for domain node (default: 2)

**Example:**
```json
{
  "jsonrpc": "2.0",
  "id": 7,
  "method": "tools/call",
  "params": {
    "name": "generate_graph",
    "arguments": {
      "browser": "all",
      "format": "json",
      "min_threshold": 2
    }
  }
}
```

## Error Handling

Errors follow JSON-RPC 2.0 specification:

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "error": {
    "code": -32603,
    "message": "Internal error: Browser not found",
    "data": null
  }
}
```

**Error Codes:**
- `-32700`: Parse error
- `-32603`: Internal error
- `-32601`: Method not found

## Integration

### Claude Desktop

Add to `claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "bookmark": {
      "command": "/path/to/bookmark-mcp"
    }
  }
}
```

### Windsurf

The MCP server can be used with Windsurf and other MCP-compatible AI assistants.

## Logging

Enable debug logging:

```bash
RUST_LOG=debug ./target/release/bookmark-mcp
```

## Security

- Read-only access to browser data
- No modification of original files
- Temporary files cleaned up after operations
- No plaintext passwords in logs
