# Bookmark Manager

[![Tests](https://img.shields.io/badge/tests-passing-brightgreen)]()
[![Rust](https://img.shields.io/badge/rust-2024-orange)]()
[![License](https://img.shields.io/badge/license-Apache--2.0-blue)]()

A Rust library and toolkit to import, search, organize, and generate knowledge graphs from browser bookmarks and history.

## Features

- **Multi-browser support**: Chrome, Firefox, Safari, Edge
- **Search and open bookmarks** instantly
- **Export bookmarks and history** to YAML
- **Remove duplicates** and organize into folders
- **Generate knowledge graphs** (DOT, JSON, GEXF formats)
- **Three usage modes**: CLI, Library API, MCP Server

## Installation

### CLI Tool (Default)
```bash
cargo build --release
./target/release/bookmark --help
```

### MCP Server
```bash
cargo build --release --features mcp --bin bookmark-mcp
./target/release/bookmark-mcp
```

### Library
Add to your `Cargo.toml`:
```toml
[dependencies]
bookmark = "0.1.2"
```

## Quick Start

```bash
# Run the demo
./demo.sh

# Or try specific examples
./examples/quick-start.sh
./examples/knowledge-graph.sh
./examples/processing.sh
./examples/search-open.sh
```

## Usage Modes

### 1. CLI Mode (Default)

Command-line interface for interactive use:

```bash
# Export bookmarks
cargo run -- export --browser chrome

# Search bookmarks
cargo run -- search github

# Generate knowledge graph
cargo run -- graph --format dot -o graph.dot
```

### 2. Library API

Use as a Rust library in your projects:

```rust
use bookmark::{BookmarkManager, Bookmark};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manager = BookmarkManager::new();
    
    // Export bookmarks
    let bookmarks = manager.export_bookmarks("chrome")?;
    
    // Search
    let results = manager.search("github")?;
    
    // Generate knowledge graph
    let graph = manager.graph_from_bookmarks(&bookmarks)?;
    
    Ok(())
}
```

See `examples/library_usage.rs` for a complete example.

### 3. MCP Server

Model Context Protocol server for AI assistants:

```bash
# Build and run MCP server
cargo build --release --features mcp --bin bookmark-mcp
./target/release/bookmark-mcp
```

**Available MCP Tools:**
- `export_bookmarks` - Export bookmarks from browsers
- `search_bookmarks` - Search bookmarks by query
- `list_browsers` - List available browsers
- `process_bookmarks` - Deduplicate and organize
- `generate_graph` - Generate knowledge graphs

## Basic Usage

```bash
# Export bookmarks
cargo run -- export

# Search bookmarks
cargo run -- search github

# Open a bookmark
cargo run -- open github

# Process (dedupe + organize)
cargo run -- process -i bookmarks.yaml -o clean.yaml

# Generate knowledge graph
cargo run -- graph --format dot -o graph.dot
```

## Commands

### `export` - Export bookmarks/history
```bash
cargo run -- export                          # Export all
cargo run -- export --browser chrome         # Specific browser
cargo run -- export --data-type history      # History only
cargo run -- export --output ./exports       # Custom output
```

### `list` - List browsers
```bash
cargo run -- list                            # All browsers
cargo run -- list --browser chrome           # Specific browser
```

### `search` - Search bookmarks
```bash
cargo run -- search github                   # Basic search
cargo run -- search docs --title-only        # Title only
cargo run -- search github --limit 10        # Limit results
```

### `open` - Open in browser
```bash
cargo run -- open github --first              # First match
cargo run -- open github                      # Interactive
```

### `process` - Dedupe and organize
```bash
cargo run -- process -i in.yaml -o out.yaml   # Both dedupe + organize
cargo run -- process --mode dedupe            # Dedupe only
cargo run -- process --mode organize          # Organize only
cargo run -- process --strategy recent        # Merge strategy
cargo run -- process --org-strategy domain    # Org strategy
cargo run -- process --preview                # Preview changes
```

### `graph` - Generate knowledge graphs
```bash
cargo run -- graph --format dot -o graph.dot  # DOT format
cargo run -- graph --format json -o graph.json # JSON format
cargo run -- graph --format gexf -o graph.gexf # GEXF format
cargo run -- graph --min-threshold 3          # Domain threshold
```

### `config` - Manage settings
```bash
cargo run -- config --show                    # Show config
cargo run -- config --list-rules              # List rules
cargo run -- config --create-sample cfg.yaml  # Create sample
```

## Knowledge Graphs

Generate visual graphs showing relationships between bookmarks:

- **Domain-based**: Links bookmarks from the same domains
- **Folder-based**: Links bookmarks in the same folders
- **Formats**: DOT (Graphviz), JSON (web), GEXF (Gephi)

```bash
# Generate DOT for Graphviz
cargo run -- graph --format dot -o bookmarks.dot
dot -Tpng bookmarks.dot -o bookmarks.png

# Generate GEXF for Gephi
cargo run -- graph --format gexf --min-threshold 3 -o analysis.gexf
```

## Safari Export

On macOS, Safari bookmarks are protected. Export manually:

1. Copy `~/Library/Safari/Bookmarks.plist` to Desktop
2. Run: `cargo run -- export --browser safari --profile-dir ~/Desktop/Bookmarks.plist`

## Options

| Option | Values |
|--------|--------|
| `--browser` | chrome, firefox, safari, edge, all |
| `--data-type` | bookmarks, history, both |
| `--format` | dot, json, gexf (graph) |
| `--mode` | dedupe, organize, both |
| `--strategy` | first, last, recent, merge |
| `--org-strategy` | domain, category, custom |

## Development

```bash
# Build all modes
cargo build --release --all-features

# Build specific modes
cargo build --release                              # CLI only
cargo build --release --features mcp --bin bookmark-mcp  # MCP server
cargo build --release --lib                        # Library only

# Test all modes
./test_all_modes.sh

# Run tests individually
cargo test --lib                    # Unit tests (39 tests)
cargo test --test integration_test  # Integration tests (3 tests)
cargo test --features mcp --test mcp_test  # MCP tests
cargo test --doc                    # Documentation tests (1 test)

# Debug logging
RUST_LOG=debug cargo run -- export
```

### Build Features

- **default**: CLI mode with `clap`, `dialoguer`, `open`
- **cli**: Command-line interface dependencies
- **mcp**: MCP server support

## Examples

See [examples/](examples/) directory for demo scripts:

- **[demo.sh](demo.sh)** - Full feature demo (interactive)
- **[examples/quick-start.sh](examples/quick-start.sh)** - Export, search, graph generation
- **[examples/knowledge-graph.sh](examples/knowledge-graph.sh)** - Graph generation (DOT, JSON, GEXF)
- **[examples/processing.sh](examples/processing.sh)** - Deduplication and organization
- **[examples/search-open.sh](examples/search-open.sh)** - Search patterns and opening

See [examples/README.md](examples/README.md) for detailed documentation.

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for version history and release notes.

## License

Apache-2.0
