use bookmark::{BookmarkManager, Bookmark};
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    
    println!("=== Bookmark Library API Demo ===\n");

    let manager = BookmarkManager::new()
        .with_export_dir(PathBuf::from("/tmp/bookmark_demo"));

    println!("1. Creating sample bookmarks...");
    let bookmarks = vec![
        Bookmark {
            id: "1".to_string(),
            title: "Rust Programming Language".to_string(),
            url: Some("https://www.rust-lang.org".to_string()),
            folder: Some("Programming".to_string()),
            date_added: None,
            children: None,
        },
        Bookmark {
            id: "2".to_string(),
            title: "GitHub".to_string(),
            url: Some("https://github.com".to_string()),
            folder: Some("Development".to_string()),
            date_added: None,
            children: None,
        },
        Bookmark {
            id: "3".to_string(),
            title: "Rust Documentation".to_string(),
            url: Some("https://doc.rust-lang.org".to_string()),
            folder: Some("Programming".to_string()),
            date_added: None,
            children: None,
        },
    ];

    println!("   Created {} bookmarks\n", bookmarks.len());

    println!("2. Generating knowledge graph...");
    match manager.graph_from_bookmarks(&bookmarks) {
        Ok(graph) => {
            println!("   ✓ Graph generated successfully");
            println!("   - Nodes: {}", graph.metadata.total_nodes);
            println!("   - Edges: {}", graph.metadata.total_edges);
            println!("   - Bookmarks: {}", graph.metadata.bookmark_count);
            println!("   - Domains: {}", graph.metadata.domain_count);
        }
        Err(e) => {
            println!("   ✗ Failed to generate graph: {}", e);
        }
    }

    println!("\n=== Demo Complete ===");
    Ok(())
}
