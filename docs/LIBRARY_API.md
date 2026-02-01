# Library API Documentation

## Overview

The Bookmark library provides a Rust API for managing browser bookmarks programmatically.

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
bookmark = "0.1.2"
```

## Core API

### BookmarkManager

Main entry point for the library.

```rust
use bookmark::BookmarkManager;

let manager = BookmarkManager::new();
```

#### Methods

##### `new() -> Self`

Create a new bookmark manager instance.

```rust
let manager = BookmarkManager::new();
```

##### `with_export_dir(dir: PathBuf) -> Self`

Set the default export directory.

```rust
use std::path::PathBuf;

let manager = BookmarkManager::new()
    .with_export_dir(PathBuf::from("/tmp/bookmarks"));
```

##### `export_bookmarks(&self, browser: &str) -> Result<Vec<Bookmark>>`

Export bookmarks from a specific browser.

**Parameters:**
- `browser`: Browser name ("chrome", "firefox", "safari", "edge")

**Returns:**
- `Result<Vec<Bookmark>>`: Vector of bookmarks or error

**Example:**
```rust
let bookmarks = manager.export_bookmarks("chrome")?;
println!("Exported {} bookmarks", bookmarks.len());
```

##### `search(&self, query: &str) -> Result<Vec<Bookmark>>`

Search bookmarks by query string.

**Parameters:**
- `query`: Search query string

**Returns:**
- `Result<Vec<Bookmark>>`: Matching bookmarks or error

**Example:**
```rust
let results = manager.search("github")?;
for bookmark in results {
    println!("{}: {}", bookmark.title, bookmark.url.unwrap_or_default());
}
```

##### `graph_from_bookmarks(&self, bookmarks: &[Bookmark]) -> Result<KnowledgeGraph>`

Generate a knowledge graph from bookmarks.

**Parameters:**
- `bookmarks`: Slice of bookmarks

**Returns:**
- `Result<KnowledgeGraph>`: Knowledge graph or error

**Example:**
```rust
let graph = manager.graph_from_bookmarks(&bookmarks)?;
println!("Graph has {} nodes and {} edges", 
    graph.metadata.total_nodes, 
    graph.metadata.total_edges);
```

## Data Types

### Bookmark

```rust
pub struct Bookmark {
    pub id: String,
    pub title: String,
    pub url: Option<String>,
    pub folder: Option<String>,
    pub date_added: Option<DateTime<Utc>>,
    pub children: Option<Vec<Bookmark>>,
}
```

### KnowledgeGraph

```rust
pub struct KnowledgeGraph {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    pub metadata: GraphMetadata,
}
```

### GraphConfig

```rust
pub struct GraphConfig {
    pub min_domain_threshold: usize,
    pub include_domain_edges: bool,
    pub include_folder_edges: bool,
    pub include_temporal_edges: bool,
}
```

## Complete Example

```rust
use bookmark::{BookmarkManager, Bookmark, GraphConfig, GraphBuilder};
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create manager with custom export directory
    let manager = BookmarkManager::new()
        .with_export_dir(PathBuf::from("/tmp/bookmarks"));

    // Export bookmarks from Chrome
    println!("Exporting Chrome bookmarks...");
    let bookmarks = manager.export_bookmarks("chrome")?;
    println!("Exported {} bookmarks", bookmarks.len());

    // Search for specific bookmarks
    println!("\nSearching for 'rust'...");
    let results = manager.search("rust")?;
    for bookmark in &results {
        println!("  - {}", bookmark.title);
    }

    // Generate knowledge graph
    println!("\nGenerating knowledge graph...");
    let graph = manager.graph_from_bookmarks(&bookmarks)?;
    println!("Graph statistics:");
    println!("  Nodes: {}", graph.metadata.total_nodes);
    println!("  Edges: {}", graph.metadata.total_edges);
    println!("  Bookmarks: {}", graph.metadata.bookmark_count);
    println!("  Domains: {}", graph.metadata.domain_count);

    // Export graph to DOT format
    use bookmark::graph::formats;
    let dot = formats::to_dot(&graph);
    std::fs::write("/tmp/bookmarks.dot", dot)?;
    println!("\nGraph exported to /tmp/bookmarks.dot");

    Ok(())
}
```

## Advanced Usage

### Custom Graph Configuration

```rust
use bookmark::{GraphConfig, GraphBuilder};

let config = GraphConfig {
    min_domain_threshold: 3,
    include_domain_edges: true,
    include_folder_edges: true,
    include_temporal_edges: false,
};

let mut builder = GraphBuilder::new(config);
let graph = builder.from_bookmarks(&bookmarks)?;
```

### Processing Bookmarks

```rust
use bookmark::processor::{BookmarkProcessor, ProcessingConfig};
use bookmark::deduplication::{DeduplicationConfig, MergeStrategy};

let config = ProcessingConfig {
    deduplication_config: DeduplicationConfig {
        merge_strategy: MergeStrategy::MergeMetadata,
        normalize_urls: true,
        ..Default::default()
    },
    ..Default::default()
};

let processor = BookmarkProcessor::new(config);
let result = processor.process_bookmarks(&bookmarks)?;

println!("Removed {} duplicates", 
    result.processing_summary.duplicates_removed);
```

## Error Handling

All methods return `Result` types. Handle errors appropriately:

```rust
match manager.export_bookmarks("chrome") {
    Ok(bookmarks) => {
        println!("Success: {} bookmarks", bookmarks.len());
    }
    Err(e) => {
        eprintln!("Error: {}", e);
    }
}
```

## Thread Safety

`BookmarkManager` is `Send` and can be used across threads. However, it is not `Sync` and should not be shared between threads without synchronization.

## Performance

- Export operations may take several seconds for large bookmark collections
- Search operations are performed in-memory and are fast
- Graph generation complexity is O(n²) for n bookmarks

## See Also

- [MCP Server Documentation](MCP_SERVER.md)
- [Examples](../examples/)
- [API Documentation](https://docs.rs/bookmark)
