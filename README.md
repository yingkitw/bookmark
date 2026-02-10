# Bookmark Manager

[![Tests](https://img.shields.io/badge/tests-passing-brightgreen)]()
[![Rust](https://img.shields.io/badge/rust-2024-orange)]()
[![License](https://img.shields.io/badge/license-Apache--2.0-blue)]()

> **Transform your browser bookmarks into interactive knowledge graphs** 🕸️

A powerful Rust toolkit that turns your browser bookmarks and history into **interactive visual knowledge graphs**. Discover hidden patterns, explore relationships between domains, and visualize your browsing activity.

## ✨ Key Feature: Knowledge Graph Generation

**Generate stunning interactive visualizations** of your bookmark collection:

- 🎨 **Interactive HTML graphs** with D3.js force-directed layouts
- 🔍 **Smart filtering**: Domains, folders, tags, categories, and bookmarks
- 🎯 **Multiple detail levels**: Overview, Standard, or Detailed views
- 🧠 **Automatic insights**: Tag extraction, categorization, similarity detection
- 📊 **Multiple formats**: HTML (interactive), DOT (Graphviz), JSON, GEXF (Gephi)
- ⚡ **Optimized for scale**: Handle 15K+ bookmarks with adaptive detail levels

### Quick Example

```bash
# Generate interactive knowledge graph (opens in browser)
cargo run --bin bookmark -- graph --format html -o graph.html

# Standard view with optimized node count
cargo run --bin bookmark -- graph --detail standard --max-per-domain 5

# Domain-only overview (fastest)
cargo run --bin bookmark -- graph --detail overview --domain-only
```

**Result**: An interactive visualization showing:
- **Bookmark nodes** (clickable to open)
- **Domain clusters** (grouped by website)
- **Folder hierarchy** (your organization structure)
- **Tag connections** (auto-extracted keywords)
- **Category grouping** (Development, Shopping, News, etc.)

![Knowledge Graph Demo](docs/images/graph-demo.png) <!-- Add screenshot if available -->

## Features

### Knowledge Graph Generation
- 🕸️ **Interactive visualizations** with D3.js (zoom, pan, drag nodes)
- 🎨 **Multiple formats**: HTML, DOT (Graphviz), JSON, GEXF (Gephi)
- 🧠 **Smart analysis**: Tag extraction, auto-categorization, similarity detection
- ⚡ **Scalable**: Optimized for large collections (15K+ bookmarks)
- 🔒 **Privacy-first**: Data processed in-memory, temp files only

### Bookmark Management
- **Multi-browser support**: Chrome, Firefox, Safari, Edge
- **Search and open** bookmarks instantly
- **Export to YAML** for backup/migration
- **Remove duplicates** intelligently
- **Auto-organize** into folders by domain/category

### Three Usage Modes
- **CLI**: Command-line tool for daily use
- **Library API**: Embed in your Rust projects
- **MCP Server**: AI assistant integration

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
cargo run --bin bookmark -- export --browser chrome

# Search bookmarks
cargo run --bin bookmark -- search github

# Generate interactive knowledge graph
cargo run --bin bookmark -- graph --format html -o graph.html

# Optimized graph for large collections
cargo run --bin bookmark -- graph --format html --detail standard --max-per-domain 5
```

### 2. Library API

Use as a Rust library in your projects:

```rust
use bookmark::{BookmarkManager, Bookmark, GraphConfig, DetailLevel};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manager = BookmarkManager::new();

    // Export bookmarks from browser
    let bookmarks = manager.export_bookmarks("chrome")?;

    // Search for specific bookmarks
    let results = manager.search("github")?;

    // Generate knowledge graph with custom config
    let config = GraphConfig {
        detail_level: DetailLevel::Standard,
        max_bookmarks_per_domain: Some(10),
        min_domain_threshold: 5,
        ..Default::default()
    };
    let graph = manager.graph_from_bookmarks_with_config(&bookmarks, config)?;

    // Export graph to different formats
    use bookmark::graph::formats;
    let html = formats::to_html_dynamic(&graph);
    let json = formats::to_json(&graph);

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
# Export bookmarks from all browsers
cargo run --bin bookmark -- export

# Search bookmarks by keyword
cargo run --bin bookmark -- search github

# Open first matching bookmark
cargo run --bin bookmark -- open github --first

# Process (dedupe + organize)
cargo run --bin bookmark -- process -i bookmarks.yaml -o clean.yaml

# Generate interactive knowledge graph
cargo run --bin bookmark -- graph --format html -o graph.html
```

## Commands

### `export` - Export bookmarks/history
```bash
cargo run --bin bookmark -- export                          # Export all
cargo run --bin bookmark -- export --browser chrome         # Specific browser
cargo run --bin bookmark -- export --data-type history      # History only
cargo run --bin bookmark -- export --output ./exports       # Custom output
```

### `list` - List browsers
```bash
cargo run --bin bookmark -- list                            # All browsers
cargo run --bin bookmark -- list --browser chrome           # Specific browser
```

### `search` - Search bookmarks
```bash
cargo run --bin bookmark -- search github                   # Basic search
cargo run --bin bookmark -- search docs --title-only        # Title only
cargo run --bin bookmark -- search github --limit 10        # Limit results
```

### `open` - Open in browser
```bash
cargo run --bin bookmark -- open github --first              # First match
cargo run --bin bookmark -- open github                      # Interactive
```

### `process` - Dedupe and organize
```bash
cargo run --bin bookmark -- process -i in.yaml -o out.yaml   # Both dedupe + organize
cargo run --bin bookmark -- process --mode dedupe            # Dedupe only
cargo run --bin bookmark -- process --mode organize          # Organize only
cargo run --bin bookmark -- process --strategy recent        # Merge strategy
cargo run --bin bookmark -- process --org-strategy domain    # Org strategy
cargo run --bin bookmark -- process --preview                # Preview changes
```

### `graph` - Generate knowledge graphs
```bash
# Basic usage
cargo run --bin bookmark -- graph --format html -o graph.html

# Performance optimization
cargo run --bin bookmark -- graph --detail standard --max-per-domain 5
cargo run --bin bookmark -- graph --detail overview --domain-only
cargo run --bin bookmark -- graph --since 2024-01-01T00:00:00Z

# Different formats
cargo run --bin bookmark -- graph --format dot -o graph.dot   # Graphviz
cargo run --bin bookmark -- graph --format json -o graph.json # JSON
cargo run --bin bookmark -- graph --format gexf -o graph.gexf # Gephi

# Advanced options
cargo run --bin bookmark -- graph --min-threshold 10           # Min bookmarks per domain
cargo run --bin bookmark -- graph --max-total 3000            # Max total nodes
```

### `config` - Manage settings
```bash
cargo run --bin bookmark -- config --show                    # Show config
cargo run --bin bookmark -- config --list-rules              # List rules
cargo run --bin bookmark -- config --create-sample cfg.yaml  # Create sample
```

## Knowledge Graph Generation

### 🎯 Why Knowledge Graphs?

Transform your flat bookmark list into a **visual exploration tool**:
- See which domains you visit most
- Discover related content through tag connections
- Understand your bookmark organization structure
- Find clusters of related bookmarks
- Identify patterns in your browsing behavior

### 📊 Graph Features

**Node Types**:
- 📌 **Bookmarks**: Your saved links (clickable to open)
- 🌐 **Domains**: Websites grouped together
- 📁 **Folders**: Your organization structure
- 🏷️ **Tags**: Auto-extracted keywords
- 📂 **Categories**: Auto-classified groups

**Edge Types**:
- Bookmark → Domain (belongs to)
- Bookmark → Folder (in folder)
- Domain ↔ Domain (same domain connections)
- Bookmark → Tag (has tag)
- Bookmark → Category (in category)
- Bookmark ↔ Bookmark (similar content)

**Analysis Features**:
- **Tag extraction**: Auto-detects keywords from titles/URLs
- **Auto-categorization**: Classifies into Development, Shopping, News, etc.
- **Similarity detection**: Finds related bookmarks using Jaccard similarity
- **Domain clustering**: Groups by website

### 🎨 Output Formats

| Format | Use Case | Tools |
|--------|----------|-------|
| **HTML** | Interactive visualization (default) | Browser |
| **DOT** | Static diagrams, printing | Graphviz |
| **JSON** | Web applications, custom viz | D3.js, Cytoscape.js |
| **GEXF** | Advanced network analysis | Gephi |

### ⚡ Performance Optimization

For large bookmark collections (10K+ items), use these options:

```bash
# Standard detail (balanced)
--detail standard --max-per-domain 5

# Overview (fastest)
--detail overview --domain-only

# Recent bookmarks only
--since 2024-01-01T00:00:00Z

# Limit total nodes
--max-total 5000

# Adjust minimum threshold
--min-threshold 10
```

| Setting | Effect | Use Case |
|---------|--------|----------|
| `--detail overview` | Domains + folders only | Quick overview |
| `--detail standard` | Top N per domain (default: 10) | Balanced view |
| `--detail detailed` | All bookmarks | Complete picture |
| `--max-per-domain N` | Limit bookmarks per domain | Reduce clutter |
| `--max-total N` | Limit total nodes | Performance |
| `--domain-only` | Skip bookmark nodes | Domain analysis |
| `--since DATE` | Only recent bookmarks | Current activity |
| `--min-threshold N` | Min bookmarks per domain | Filter noise |

### 🔒 Privacy & Security

**Your data stays private**:
- ✅ All processing happens **in-memory**
- ✅ Graph data stored in **system temp** folder
- ✅ No personal data in your project directory
- ✅ Temp files auto-cleaned by OS

```bash
# Files created in temp (not in your project)
/var/folders/.../T/bookmark-graph/graph_20260210.html
/var/folders/.../T/bookmark-graph/graph_20260210.data.js
```

### 🎮 Interactive HTML Features

The HTML visualization includes:
- **Force-directed layout** with physics simulation
- **Zoom & pan** for navigation
- **Drag nodes** to rearrange
- **Dark/light theme** toggle
- **Node type filters** (show/hide types)
- **Adjustable physics** (charge, distance)
- **Hover tooltips** with details
- **Click-to-open** bookmarks

### 📝 Examples

```bash
# Quick interactive graph (recommended)
cargo run --bin bookmark -- graph --format html -o graph.html

# Generate DOT for Graphviz (customizable)
cargo run --bin bookmark -- graph --format dot -o bookmarks.dot
dot -Tpng bookmarks.dot -o bookmarks.png

# Generate GEXF for Gephi analysis
cargo run --bin bookmark -- graph --format gexf --min-threshold 3 -o analysis.gexf

# JSON for custom visualization
cargo run --bin bookmark -- graph --format json -o data.json
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
| `--format` | html, dot, json, gexf (graph) |
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
cargo test --lib                    # Unit tests (45 tests)
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
- **[examples/knowledge-graph.sh](examples/knowledge-graph.sh)** - Graph generation with multiple formats
- **[examples/processing.sh](examples/processing.sh)** - Deduplication and organization
- **[examples/search-open.sh](examples/search-open.sh)** - Search patterns and opening

### Knowledge Graph Example

```bash
#!/bin/bash
# Generate optimized knowledge graph

# 1. Export bookmarks from all browsers
cargo run --bin bookmark -- export --browser all

# 2. Generate interactive HTML (opens in browser)
cargo run --bin bookmark -- graph \
  --format html \
  --detail standard \
  --max-per-domain 5 \
  -o graph.html

# 3. Files are created in temp directory (not in your project!)
# HTML opens automatically in your default browser

# 4. For analysis, generate GEXF for Gephi
cargo run --bin bookmark -- graph \
  --format gexf \
  --min-threshold 10 \
  -o analysis.gexf
```

See [examples/README.md](examples/README.md) for detailed documentation.

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for version history and release notes.

## License

Apache-2.0
